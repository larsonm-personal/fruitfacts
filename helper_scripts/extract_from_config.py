#!/usr/bin/env python3
"""Extract a draft JSON5 reference from a parser config"""

import json
import re
import sys
from pathlib import Path

from fruitfacts_extract.html_tools import blocks_between_headings
from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.pdf_tools import clean_pdf_text
from fruitfacts_extract.pdf_tools import clean_pdf_layout_text
from fruitfacts_extract.pdf_tools import pdf_url_to_text
from fruitfacts_extract.pdf_table_tools import fixed_width_table_rows
from fruitfacts_extract.pdf_table_tools import catalog_entry_rows
from fruitfacts_extract.pdf_table_tools import line_matches_any
from fruitfacts_extract.pdf_table_tools import numbered_block_rows
from fruitfacts_extract.pdf_table_tools import pdf_bullet_list_rows
from fruitfacts_extract.pdf_table_tools import pdf_quoted_entry_rows
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import category_records
from fruitfacts_extract.record_tools import labelled_description_from_row
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.record_tools import strip_trailing_note_markers
from fruitfacts_extract.table_tools import fill_leading_group_cells
from fruitfacts_extract.table_tools import find_table
from fruitfacts_extract.table_tools import find_table_with_header_row
from fruitfacts_extract.table_tools import keyed_data_rows
from fruitfacts_extract.table_tools import merge_leading_fragment_rows
from fruitfacts_extract.table_tools import table_to_dicts
from fruitfacts_extract.table_tools import table_to_dicts_with_sections
from fruitfacts_extract.text_tools import clean_text
from fruitfacts_extract.text_tools import first_sentence_containing
from fruitfacts_extract.text_tools import names_after_marker
from fruitfacts_extract.text_tools import quoted_name_paragraph
from fruitfacts_extract.text_tools import section_between
from fruitfacts_extract.text_tools import split_sentences


def load_config(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def merged_extractor(config, extractor):
    merged = dict(extractor)
    text_fixes = dict(config.get("text_fixes", {}))
    text_fixes.update(extractor.get("text_fixes", {}))
    if text_fixes:
        merged["text_fixes"] = text_fixes
    return merged


def pairs(data):
    return [(item[0], item[1]) for item in data]


def name_overrides(config):
    return config.get("name_overrides", {})


def source_data(config):
    source = config["source"]
    if source["kind"] == "html":
        return fetch_html_page(source["url"])
    if source["kind"] == "pdf":
        pdf_text = pdf_url_to_text(source["url"], layout=source.get("layout", True))
        if source.get("layout_text"):
            return clean_pdf_layout_text(pdf_text)
        if source.get("line_text"):
            return clean_pdf_text(pdf_text)
        return clean_text(clean_pdf_text(pdf_text))
    raise ValueError("Unsupported source kind: " + source["kind"])


def raw_table(page, extractor):
    if "table_index" in extractor:
        tables = [page.tables[extractor["table_index"]]]
        if extractor.get("header_row"):
            return find_table_with_header_row(
                tables,
                extractor["required_headers"],
                title_contains=extractor.get("title_contains"),
            )
        return find_table(tables, extractor["required_headers"])
    if extractor.get("header_row"):
        return find_table_with_header_row(
            page.tables,
            extractor["required_headers"],
            title_contains=extractor.get("title_contains"),
        )
    return find_table(page.tables, extractor["required_headers"])


def transformed_table(table, transforms):
    for transform in transforms:
        if transform == "fill_leading_group_cells":
            table = fill_leading_group_cells(table)
        elif transform == "merge_leading_fragment_rows":
            table = merge_leading_fragment_rows(table)
        else:
            raise ValueError("Unsupported table transform: " + transform)
    return table


def row_text_fixes(row, fixes):
    if not fixes:
        return row
    return {
        key: apply_text_fixes(value, fixes) if isinstance(value, str) else value
        for key, value in row.items()
    }


def row_overrides(row, extractor):
    key = row.get(extractor.get("name_key", "name"))
    override = extractor.get("row_overrides", {}).get(key)
    if not override:
        return row
    new_row = dict(row)
    new_row.update(override)
    return new_row


def dedupe_rows(rows, keys):
    if not keys:
        return rows
    seen = set()
    deduped = []
    for row in rows:
        signature = tuple(row.get(key) for key in keys)
        if signature in seen:
            continue
        seen.add(signature)
        deduped.append(row)
    return deduped


def apply_text_fixes(text, fixes):
    for old, new in fixes.items():
        text = text.replace(old, new)
    return text


def expanded_rows(rows, extractor):
    for row in rows:
        matched = False
        for split in extractor.get("row_splits", []):
            if all(row.get(key) == value for key, value in split["match"].items()):
                matched = True
                for replacement in split["rows"]:
                    new_row = dict(row)
                    new_row.update(replacement)
                    if split.get("source_note"):
                        new_row["_source_note"] = split["source_note"]
                    yield new_row
                break
        if not matched:
            yield row


def table_rows(page, extractor):
    table = raw_table(page, extractor)
    if extractor.get("header_row"):
        rows = table["rows"]
    else:
        transforms = extractor.get("table_transforms", [])
        if isinstance(transforms, str):
            transforms = [transforms]
        table = transformed_table(table, transforms)
        if extractor.get("section_rows"):
            rows = table_to_dicts_with_sections(
                table,
                default_section=extractor.get("default_section"),
            )
        else:
            rows = table_to_dicts(table)
    rows = expanded_rows(rows, extractor)
    rows = [row_text_fixes(row, extractor.get("text_fixes")) for row in rows]
    rows = [row_overrides(row, extractor) for row in rows]
    rows = keyed_data_rows(
        rows,
        extractor["name_key"],
        skip_prefixes=tuple(extractor.get("skip_prefixes", ["Note:"])),
    )
    skip_names = set(extractor.get("skip_names", []))
    if skip_names:
        rows = [
            row
            for row in rows
            if row.get(extractor["name_key"], "").strip() not in skip_names
        ]
    return rows


def resolved_value(spec, row):
    if spec is None:
        return None
    if isinstance(spec, str):
        return spec
    if "row_key" in spec:
        value = row.get(spec["row_key"])
        if "map" in spec and value in spec["map"]:
            return spec["map"][value]
        for prefix, mapped in spec.get("prefix_map", {}).items():
            if isinstance(value, str) and value.startswith(prefix):
                return mapped
        return spec.get("default", value)
    if "template" in spec:
        return format_template(spec["template"], row, None)
    raise ValueError("Unsupported value spec: " + repr(spec))


def format_template(template, row, extractor):
    values = dict(row)
    if extractor:
        category = resolved_value(extractor.get("category"), row)
        if category:
            values["category"] = category
            values["category_lower"] = category.lower()
    return template.format(**values)


def description_key_part(row, part):
    value = row.get(part["key"])
    if not value:
        return None
    value = apply_text_fixes(value, part.get("text_fixes", {}))
    if part.get("rstrip_period", True):
        value = value.rstrip(".")
    return value


def lookup_labels_part(row, part, lookups):
    key_value = row.get(part.get("row_key", "name"))
    lookup_row = lookups.get(part["lookup"], {}).get(key_value)
    if not lookup_row:
        return None
    values = []
    for key, label in pairs(part["labels"]):
        value = lookup_row.get(key)
        if value:
            values.append(label + " " + value)
    if not values:
        return None
    return part.get("prefix", "") + "; ".join(values)


def lookup_key_part(row, part, lookups):
    key_value = row.get(part.get("row_key", "name"))
    lookup_row = lookups.get(part["lookup"], {}).get(key_value)
    if not lookup_row:
        return None
    value = lookup_row.get(part["key"])
    if not value:
        return None
    return part.get("prefix", "") + value


def flag_labels_part(row, part):
    values = []
    for key, label in pairs(part["labels"]):
        if row.get(key):
            values.append(label)
    if not values:
        return None
    return part.get("prefix", "") + part.get("separator", "; ").join(values)


def description_part(row, extractor, part, lookups):
    kind = part["kind"]
    if kind == "labels":
        return labelled_description_from_row(row, pairs(part["labels"]))
    if kind == "key":
        return description_key_part(row, part)
    if kind == "template":
        return format_template(part["template"], row, extractor)
    if kind == "lookup_labels":
        return lookup_labels_part(row, part, lookups)
    if kind == "lookup_key":
        return lookup_key_part(row, part, lookups)
    if kind == "flag_labels":
        return flag_labels_part(row, part)
    raise ValueError("Unsupported description part: " + kind)


def description(row, extractor, lookups):
    if extractor.get("description_parts"):
        parts = [
            description_part(row, extractor, part, lookups)
            for part in extractor["description_parts"]
        ]
        description_value = extractor.get("description_joiner", ". ").join(
            part for part in parts if part
        )
    else:
        parts = [
            labelled_description_from_row(
                row,
                pairs(extractor.get("description_labels", [])),
            )
        ]
        if extractor.get("description_key"):
            parts.append(
                description_key_part(row, {"key": extractor["description_key"]})
            )
        for key in extractor.get("description_keys", []):
            parts.append(description_key_part(row, {"key": key}))
        if extractor.get("description_template"):
            parts.append(format_template(extractor["description_template"], row, extractor))
        description_value = ". ".join(part for part in parts if part)
    if extractor.get("description_suffix"):
        description_value = append_source_note(
            description_value,
            extractor["description_suffix"],
        )
    return description_value


def harvest_time(row, extractor, lookups):
    if extractor.get("harvest_time_unparsed"):
        return resolved_value(extractor["harvest_time_unparsed"], row)
    if extractor.get("harvest_phrase_map"):
        text = row.get(extractor.get("harvest_from_key", "description"), "")
        lowered = text.lower()
        for needle, value in pairs(extractor["harvest_phrase_map"]):
            if needle.lower() in lowered:
                return value
        if extractor.get("harvest_phrase_only"):
            return None
    if extractor.get("harvest_lookup"):
        lookup = extractor["harvest_lookup"]
        key_value = row.get(lookup.get("row_key", "name"))
        lookup_row = lookups.get(lookup["lookup"], {}).get(key_value)
        if lookup_row:
            return lookup_row.get(lookup["key"])
    if extractor.get("harvest_key"):
        return row.get(extractor["harvest_key"])
    if extractor.get("harvest_from_key"):
        text = row.get(extractor.get("harvest_from_key", "description"), "")
        if extractor.get("harvest_term_priority"):
            if extractor.get("harvest_term_prefix"):
                sentences = split_sentences(text)
                terms = [
                    term.lower()
                    for term in extractor.get("harvest_terms", ["ripen", "matur"])
                ]
                for term in terms:
                    for sentence in sentences:
                        if sentence.lower().startswith(term):
                            return sentence
                return None
            for term in extractor.get("harvest_terms", ["ripen", "matur"]):
                value = first_sentence_containing(text, [term])
                if value:
                    return value
            return None
        return first_sentence_containing(
            text,
            extractor.get("harvest_terms", ["ripen", "matur"]),
        )
    return None


def combined_note(*notes):
    return ". ".join(note for note in notes if note)


def normalized_source_name(source_name, overrides):
    if source_name not in overrides:
        return source_name, None, {}
    target = overrides[source_name]
    if isinstance(target, dict):
        extra = {}
        if target.get("aka"):
            extra["AKA"] = target["aka"]
        return target["name"], target.get("note"), extra
    name, note = target
    return name, note, {}


def source_name_and_note(row, extractor, overrides):
    source_name = row[extractor["name_key"]]
    if extractor.get("trim_name"):
        source_name = source_name.strip()
    notes = []
    for suffix_note in extractor.get("name_suffix_notes", []):
        suffix = suffix_note["suffix"]
        if source_name.endswith(suffix):
            source_name = source_name[: -len(suffix)].strip()
            notes.append(suffix_note["note"])
    if extractor.get("strip_name_footnotes"):
        source_name = strip_trailing_note_markers(source_name)
    name, source_note, extra_fields = normalized_source_name(source_name, overrides)
    notes.append(source_note)
    notes.append(row.get("_source_note"))
    return name, combined_note(*notes), extra_fields


def plant_records_from_config_rows(rows, extractor, overrides, lookups):
    plants = []
    for row in rows:
        name, source_note, extra_fields = source_name_and_note(row, extractor, overrides)
        record = plant_record(
            resolved_value(extractor["plant_type"], row),
            name,
            category=resolved_value(extractor.get("category"), row),
        )
        for key, value in extra_fields.items():
            record[key] = value
        harvest = harvest_time(row, extractor, lookups)
        if harvest:
            record["harvest_time_unparsed"] = harvest
        record_description = append_source_note(
            description(row, extractor, lookups),
            source_note,
        )
        if record_description:
            record["description"] = record_description
        plants.append(record)
    return plants


def plants_from_html_table(page, extractor, overrides, lookups):
    return plant_records_from_config_rows(
        table_rows(page, extractor),
        extractor,
        overrides,
        lookups,
    )


def html_rule_matches(tag, text, state, rule):
    if rule.get("tag") and tag != rule["tag"]:
        return False
    if rule.get("tags") and tag not in rule["tags"]:
        return False
    if rule.get("text") and text != rule["text"]:
        return False
    if rule.get("texts") and text not in rule["texts"]:
        return False
    if rule.get("prefix") and not text.startswith(rule["prefix"]):
        return False
    if rule.get("section") and state.get("section") != rule["section"]:
        return False
    if rule.get("subsection") and state.get("subsection") != rule["subsection"]:
        return False
    return True


def html_rule_values(rule):
    keys = {
        "tag",
        "tags",
        "text",
        "texts",
        "prefix",
        "section",
        "subsection",
    }
    return {key: value for key, value in rule.items() if key not in keys}


def apply_html_rules(tag, text, state, rules):
    for rule in rules:
        if html_rule_matches(tag, text, state, rule):
            state.update(html_rule_values(rule))
            return True
    return False


def parse_html_list_item(text, extractor):
    pattern = extractor.get(
        "item_pattern",
        r"^(?P<name>[^:]+):\s*(?P<description>.+)$",
    )
    match = re.match(pattern, text)
    if not match:
        return None
    row = {}
    for key in ("name", "description"):
        value = match.groupdict().get(key)
        if value:
            row[extractor.get(key + "_key", key)] = value.strip()
    return row


def html_list_item_rows(page, extractor):
    rows = []
    started = not extractor.get("start_after_text")
    state = {
        "section": None,
        "subsection": None,
        "category": None,
        "plant_type": None,
    }

    for tag, text in page.blocks:
        if not started:
            if text == extractor["start_after_text"]:
                started = True
            continue

        if tag.startswith("h") and text in extractor.get("stop_headings", []):
            break

        if tag == "h2":
            state = {
                "section": text,
                "subsection": None,
                "category": None,
                "plant_type": None,
            }
            apply_html_rules(tag, text, state, extractor.get("section_rules", []))
            continue

        if tag == "h3":
            state["subsection"] = text
            apply_html_rules(tag, text, state, extractor.get("subheading_rules", []))
            continue

        if tag.startswith("h"):
            apply_html_rules(tag, text, state, extractor.get("subheading_rules", []))
            continue

        if tag != "li" or not state.get("category") or not state.get("plant_type"):
            continue
        if line_matches_any(text, extractor.get("skip_item_patterns", [])):
            continue

        row = parse_html_list_item(text, extractor)
        if not row:
            continue
        row.update(
            {
                "section": state.get("section"),
                "subsection": state.get("subsection"),
                "category": state.get("category"),
                "plant_type": state.get("plant_type"),
            }
        )
        rows.append(row)

    return rows


def plants_from_html_list_items(page, extractor, overrides, lookups):
    extractor = {
        "name_key": "name",
        "description_key": "description",
        "category": {"row_key": "category"},
        "plant_type": {"row_key": "plant_type"},
        **extractor,
    }
    rows = html_list_item_rows(page, extractor)
    rows = expanded_rows(rows, extractor)
    rows = [row_text_fixes(row, extractor.get("row_text_fixes")) for row in rows]
    rows = [row_overrides(row, extractor) for row in rows]
    rows = dedupe_rows(rows, extractor.get("dedupe_keys", []))
    return plant_records_from_config_rows(rows, extractor, overrides, lookups)


def suffix_mapped_row(row, extractor):
    name_key = extractor.get("name_key", "name")
    name = row.get(name_key, "")
    for rule in extractor.get("name_suffix_type_map", []):
        suffix = rule["suffix"]
        if not name.endswith(suffix):
            continue
        row = dict(row)
        row[name_key] = name[: -len(suffix)].strip()
        for key, value in rule.items():
            if key != "suffix":
                row[key] = value
        return row
    return row


def html_name_matrix_rows(page, extractor):
    name_key = extractor.get("name_key", "name")
    table = page.tables[extractor["table_index"]]
    rows = []
    group_values = {}
    skip_patterns = extractor.get("skip_cell_patterns", [])
    skip_cells = set(extractor.get("skip_cells", []))
    group_cells = extractor.get("group_cells", {})

    for source_row in table:
        for cell in source_row:
            cell = clean_text(
                apply_text_fixes(cell, extractor.get("cell_text_fixes", {}))
            )
            if not cell or cell in skip_cells or line_matches_any(cell, skip_patterns):
                continue
            if cell in group_cells:
                group_values = dict(group_cells[cell])
                continue
            row = {name_key: cell}
            row.update(extractor.get("row_fields", {}))
            row.update(group_values)
            rows.append(suffix_mapped_row(row, extractor))

    return rows


def plants_from_html_name_matrix(page, extractor, overrides, lookups):
    extractor = {"name_key": "name", **extractor}
    rows = html_name_matrix_rows(page, extractor)
    rows = expanded_rows(rows, extractor)
    rows = [row_text_fixes(row, extractor.get("row_text_fixes")) for row in rows]
    rows = [row_overrides(row, extractor) for row in rows]
    rows = dedupe_rows(rows, extractor.get("dedupe_keys", []))
    return plant_records_from_config_rows(rows, extractor, overrides, lookups)


def pdf_marker_list_rows(text, extractor):
    section = text
    if extractor.get("section_start"):
        section = section_between(
            text,
            extractor["section_start"],
            extractor["section_end"],
        )
    rows = []
    for name in names_after_marker(
        section,
        extractor["marker"],
        extractor.get("end_marker", "."),
    ):
        rows.append({extractor.get("name_key", "name"): name})
    return rows


def plants_from_pdf_marker_list(text, extractor, overrides, lookups):
    extractor = {"name_key": "name", **extractor}
    return plant_records_from_config_rows(
        pdf_marker_list_rows(text, extractor),
        extractor,
        overrides,
        lookups,
    )


def plants_from_pdf_fixed_width_table(text, extractor, overrides, lookups):
    extractor = {"name_key": "name", **extractor}
    rows = fixed_width_table_rows(text, extractor)
    rows = expanded_rows(rows, extractor)
    rows = [row_text_fixes(row, extractor.get("text_fixes")) for row in rows]
    rows = [row_overrides(row, extractor) for row in rows]
    return plant_records_from_config_rows(
        rows,
        extractor,
        overrides,
        lookups,
    )


def plants_from_pdf_numbered_blocks(text, extractor, overrides, lookups):
    extractor = {"name_key": "name", **extractor}
    rows = numbered_block_rows(text, extractor)
    rows = expanded_rows(rows, extractor)
    rows = [row_text_fixes(row, extractor.get("text_fixes")) for row in rows]
    rows = [row_overrides(row, extractor) for row in rows]
    return plant_records_from_config_rows(
        rows,
        extractor,
        overrides,
        lookups,
    )


def plants_from_pdf_catalog_entries(text, extractor, overrides, lookups):
    extractor = {
        "name_key": "name",
        "description_key": "description",
        "category": {"row_key": "category"},
        "plant_type": {"row_key": "plant_type"},
        **extractor,
    }
    rows = catalog_entry_rows(text, extractor)
    rows = expanded_rows(rows, extractor)
    rows = [row_text_fixes(row, extractor.get("row_text_fixes")) for row in rows]
    rows = [row_overrides(row, extractor) for row in rows]
    return plant_records_from_config_rows(
        rows,
        extractor,
        overrides,
        lookups,
    )


def plants_from_pdf_bullet_list(text, extractor, overrides, lookups):
    extractor = {
        "name_key": "name",
        "description_key": "description",
        "category": {"row_key": "category"},
        "plant_type": {"row_key": "plant_type"},
        **extractor,
    }
    rows = pdf_bullet_list_rows(text, extractor)
    rows = expanded_rows(rows, extractor)
    rows = [row_text_fixes(row, extractor.get("row_text_fixes")) for row in rows]
    rows = [row_overrides(row, extractor) for row in rows]
    return plant_records_from_config_rows(
        rows,
        extractor,
        overrides,
        lookups,
    )


def plants_from_pdf_quoted_entries(text, extractor, overrides, lookups):
    extractor = {
        "name_key": "name",
        "description_key": "description",
        "category": {"row_key": "category"},
        "plant_type": {"row_key": "plant_type"},
        **extractor,
    }
    rows = pdf_quoted_entry_rows(text, extractor)
    rows = expanded_rows(rows, extractor)
    rows = [row_text_fixes(row, extractor.get("row_text_fixes")) for row in rows]
    rows = [row_overrides(row, extractor) for row in rows]
    return plant_records_from_config_rows(
        rows,
        extractor,
        overrides,
        lookups,
    )


def quoted_paragraph_rows(page, extractor):
    blocks = blocks_between_headings(
        page.blocks,
        extractor["start_heading"],
        extractor.get("end_heading"),
    )
    category = extractor.get("default_category")
    rows = []
    for tag, text in blocks:
        for switch in extractor.get("category_switches", []):
            if text == switch["text"]:
                category = switch["category"]
                break
        if tag not in extractor.get("paragraph_tags", ["p"]):
            continue
        if not text.startswith(extractor.get("quote_prefix", "'")):
            continue
        name, description_value = quoted_name_paragraph(text)
        if extractor.get("prepend_name_if_lowercase") and description_value:
            if description_value[0].islower():
                description_value = name + " " + description_value
        row = {
            "name": name,
            "category": category,
            "description": description_value,
        }
        if extractor.get("harvest_terms"):
            row["harvest_time_unparsed"] = first_sentence_containing(
                description_value,
                extractor["harvest_terms"],
            )
        rows.append(row)
    return rows


def plants_from_quoted_paragraphs(page, extractor, overrides, lookups):
    extractor = {"name_key": "name", **extractor}
    return plant_records_from_config_rows(
        quoted_paragraph_rows(page, extractor),
        extractor,
        overrides,
        lookups,
    )


def matches_category_rule(tag, text, rule):
    if rule.get("tags") and tag not in rule["tags"]:
        return False
    if rule.get("texts") and text not in rule["texts"]:
        return False
    return True


def category_from_text(text, extractor):
    category = apply_text_fixes(text, extractor.get("category_text_fixes", {}))
    if extractor.get("strip_category_parenthetical"):
        match = re.match(r"(.+?)\s+\((.+)\)$", category)
        if match:
            category = match.group(1)
    return category


def generated_rows_for_text(text, extractor):
    rows = []
    for generated in extractor.get("generated_rows", []):
        if not text.startswith(generated["text_prefix"]):
            continue
        rows.extend(generated["rows"])
    return rows


def paragraph_match(text, extractor):
    pattern = extractor.get("paragraph_pattern", r"([^:]{2,80}):\s+(.+)$")
    match = re.match(pattern, text)
    if not match:
        return None
    return match


def description_from_following(blocks, index, extractor):
    if not extractor.get("description_from_following"):
        return None
    if index + 1 >= len(blocks):
        return None
    next_tag, next_text = blocks[index + 1]
    if next_tag != "p":
        return None
    if re.match(extractor.get("following_disqualify_pattern", r"[^:]{2,80}:\s*"), next_text):
        return None
    return next_text


def colon_paragraph_rows(page, extractor):
    rows = []
    current_category = None
    started = not extractor.get("start_after_text")
    skip_index = None
    for index, (tag, text) in enumerate(page.blocks):
        if index == skip_index:
            continue
        if not started:
            if text == extractor["start_after_text"]:
                started = True
            continue
        if current_category and any(
            text.startswith(prefix) for prefix in extractor.get("stop_prefixes", [])
        ):
            break
        if any(matches_category_rule(tag, text, rule) for rule in extractor["category_rules"]):
            current_category = category_from_text(text, extractor)
            continue
        if not current_category or tag != "p":
            continue
        generated_rows = generated_rows_for_text(text, extractor)
        if generated_rows:
            rows.extend(generated_rows)
            continue
        match = paragraph_match(text, extractor)
        if not match:
            continue
        name = match.group("name") if "name" in match.groupdict() else match.group(1)
        description_value = (
            match.group("description")
            if "description" in match.groupdict()
            else match.group(2)
        )
        description_value = (description_value or "").strip()
        if not description_value:
            description_value = description_from_following(page.blocks, index, extractor)
            if description_value:
                skip_index = index + 1
        if not description_value:
            continue
        rows.append(
            {
                "name": name.strip(),
                "category": current_category,
                "description": description_value,
            }
        )
    return rows


def plants_from_colon_paragraphs(page, extractor, overrides, lookups):
    extractor = {"name_key": "name", **extractor}
    return plant_records_from_config_rows(
        colon_paragraph_rows(page, extractor),
        extractor,
        overrides,
        lookups,
    )


def build_lookups(data, config):
    lookups = {}
    for lookup in config.get("lookups", []):
        if lookup["kind"] != "html_table":
            raise ValueError("Unsupported lookup kind: " + lookup["kind"])
        rows = table_rows(data, lookup)
        lookups[lookup["name"]] = {row[lookup["key"]]: row for row in rows}
    return lookups


def extract(config):
    data = source_data(config)
    overrides = name_overrides(config)
    lookups = build_lookups(data, config)
    plants = []
    for source_extractor in config["extractors"]:
        extractor = merged_extractor(config, source_extractor)
        if extractor["kind"] == "html_table":
            plants.extend(plants_from_html_table(data, extractor, overrides, lookups))
        elif extractor["kind"] == "html_list_items":
            plants.extend(plants_from_html_list_items(data, extractor, overrides, lookups))
        elif extractor["kind"] == "html_name_matrix":
            plants.extend(plants_from_html_name_matrix(data, extractor, overrides, lookups))
        elif extractor["kind"] == "pdf_marker_list":
            plants.extend(plants_from_pdf_marker_list(data, extractor, overrides, lookups))
        elif extractor["kind"] == "pdf_fixed_width_table":
            plants.extend(
                plants_from_pdf_fixed_width_table(data, extractor, overrides, lookups)
            )
        elif extractor["kind"] == "pdf_numbered_blocks":
            plants.extend(
                plants_from_pdf_numbered_blocks(data, extractor, overrides, lookups)
            )
        elif extractor["kind"] == "pdf_catalog_entries":
            plants.extend(
                plants_from_pdf_catalog_entries(data, extractor, overrides, lookups)
            )
        elif extractor["kind"] == "pdf_bullet_list":
            plants.extend(
                plants_from_pdf_bullet_list(data, extractor, overrides, lookups)
            )
        elif extractor["kind"] == "pdf_quoted_entries":
            plants.extend(
                plants_from_pdf_quoted_entries(data, extractor, overrides, lookups)
            )
        elif extractor["kind"] == "quoted_paragraph_blocks":
            plants.extend(
                plants_from_quoted_paragraphs(data, extractor, overrides, lookups)
            )
        elif extractor["kind"] == "colon_paragraph_blocks":
            plants.extend(plants_from_colon_paragraphs(data, extractor, overrides, lookups))
        else:
            raise ValueError("Unsupported extractor kind: " + extractor["kind"])
    return plants


def emit_config(path):
    config = load_config(path)
    emit_reference(
        pairs(config["reference_fields"]),
        extract(config),
        categories=category_records(config.get("categories", [])),
        locations=config.get("locations"),
    )


def main(argv):
    if len(argv) != 2:
        print("Usage: extract_from_config.py CONFIG.json", file=sys.stderr)
        return 2
    try:
        emit_config(argv[1])
    except Exception as error:
        print(f"Failed to extract from config: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
