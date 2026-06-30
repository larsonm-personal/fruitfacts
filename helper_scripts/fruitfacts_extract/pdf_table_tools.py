"""Helpers for table-like text extracted from PDFs"""

import re

from fruitfacts_extract.text_tools import clean_text
from fruitfacts_extract.text_tools import section_between


def table_section(text, extractor):
    section = text
    if extractor.get("section_start"):
        if extractor.get("section_end"):
            section = section_between(
                text,
                extractor["section_start"],
                extractor["section_end"],
            )
        else:
            start_index = text.find(extractor["section_start"])
            if start_index < 0:
                raise ValueError("Could not find section start: " + extractor["section_start"])
            section = text[start_index:]
    return section


def line_matches_any(line, patterns):
    return any(re.search(pattern, line) for pattern in patterns)


def fullmatch(pattern, value):
    return re.fullmatch(pattern, value or "") is not None


def split_suffix(row, transform):
    value = row.get(transform["source_key"], "")
    for suffix in sorted(transform["suffixes"], key=len, reverse=True):
        pattern = r"^(.*?)\s+" + re.escape(suffix) + r"$"
        match = re.match(pattern, value)
        if not match:
            continue
        row[transform["left_key"]] = match.group(1).strip()
        row[transform["right_key"]] = suffix
        return row
    row[transform["left_key"]] = value
    row[transform["right_key"]] = ""
    return row


def split_on_token(row, transform):
    value = row.get(transform["source_key"], "")
    for token in sorted(transform["tokens"], key=len, reverse=True):
        pattern = r"^(.*?)\s+" + re.escape(token) + r"(?:\s+(.*))?$"
        match = re.match(pattern, value)
        if not match:
            continue
        row[transform["left_key"]] = match.group(1).strip()
        row[transform["right_key"]] = token
        remainder = (match.group(2) or "").strip()
        if remainder and transform.get("remainder_key"):
            target_key = transform["remainder_key"]
            existing = row.get(target_key, "")
            joiner = transform.get("remainder_joiner", " ")
            row[target_key] = (remainder + joiner + existing).strip()
        return row
    row[transform["left_key"]] = value
    row[transform["right_key"]] = ""
    return row


def split_prefix_token(row, transform):
    value = row.get(transform["source_key"], "")
    for token in sorted(transform["tokens"], key=len, reverse=True):
        pattern = r"^" + re.escape(token) + r"(?:\s+(.*))?$"
        match = re.match(pattern, value)
        if not match:
            continue
        row[transform["left_key"]] = token
        row[transform["right_key"]] = (match.group(1) or "").strip()
        return row
    row[transform["left_key"]] = ""
    row[transform["right_key"]] = value
    return row


def apply_row_transforms(row, transforms):
    for transform in transforms:
        if transform["kind"] == "split_suffix":
            row = split_suffix(row, transform)
        elif transform["kind"] == "split_on_token":
            row = split_on_token(row, transform)
        elif transform["kind"] == "split_prefix_token":
            row = split_prefix_token(row, transform)
        else:
            raise ValueError("Unsupported PDF table row transform: " + transform["kind"])
    return row


def smart_title_token(token, preserve_upper_words):
    if token in preserve_upper_words:
        return token
    if any(character.islower() for character in token):
        return token
    if any(character.isdigit() for character in token):
        return token
    return token[:1].upper() + token[1:].lower()


def smart_title_name(name, preserve_upper_words=None):
    preserve_upper_words = set(preserve_upper_words or [])
    return " ".join(
        smart_title_token(token, preserve_upper_words) for token in name.split()
    )


def parse_catalog_entry(line, extractor):
    name_key = extractor.get("name_key", "name")
    description_key = extractor.get("description_key", "description")
    for name in sorted(extractor.get("entry_names", []), key=len, reverse=True):
        if line == name:
            return None
        if line.startswith(name + " "):
            return {
                name_key: name,
                description_key: line[len(name) :].strip(),
            }

    tokens = line.split()
    if not tokens:
        return None
    description_starters = set(
        extractor.get(
            "description_start_words",
            [
                "A",
                "An",
                "Another",
                "Bred",
                "Developed",
                "First",
                "From",
                "Fruit",
                "Introduced",
                "Large",
                "Medium",
                "Moderate",
                "Originating",
                "Produces",
                "Released",
                "Similar",
                "Small",
                "Sweetest",
                "The",
                "These",
                "This",
                "Vigorous",
                "Winter-hardy",
            ],
        )
    )
    name_tokens = []
    description_tokens = []
    for index, token in enumerate(tokens):
        if not name_tokens and token in description_starters:
            return None
        if name_tokens and (
            token in description_starters or any(character.islower() for character in token)
        ):
            description_tokens = tokens[index:]
            break
        if any(character.islower() for character in token):
            return None
        name_tokens.append(token)

    if not name_tokens or not description_tokens:
        return None
    return {
        name_key: " ".join(name_tokens),
        description_key: " ".join(description_tokens),
    }


def catalog_category_for_line(line, extractor):
    for rule in extractor.get("category_rules", []):
        if rule.get("text") and line == rule["text"]:
            return rule, ""
        prefix = rule.get("prefix")
        if prefix and line.startswith(prefix):
            return rule, line[len(prefix) :].strip()
    return None, None


def rule_context(rule):
    control_keys = {
        "text",
        "prefix",
        "parse_remainder",
        "groups",
        "split_before",
    }
    return {key: value for key, value in rule.items() if key not in control_keys}


def initial_context(extractor):
    context = dict(extractor.get("default_context", {}))
    if extractor.get("default_category"):
        context["category"] = extractor["default_category"]
    if extractor.get("default_plant_type"):
        context["plant_type"] = extractor["default_plant_type"]
    return context or None


def finish_catalog_row(row, extractor):
    if extractor.get("name_case") == "smart_title":
        row[extractor.get("name_key", "name")] = smart_title_name(
            row[extractor.get("name_key", "name")],
            preserve_upper_words=extractor.get("preserve_upper_words"),
        )
    return row


def catalog_entry_rows(text, extractor):
    lines = section_lines(text, extractor)
    rows = []
    current_category = initial_context(extractor)
    current = None
    name_key = extractor.get("name_key", "name")
    description_key = extractor.get("description_key", "description")
    for line in lines:
        line = clean_text(line)
        line = apply_text_fixes_to_line(line, extractor.get("text_fixes", {}))
        if line_matches_any(line, extractor.get("skip_line_patterns", [])):
            continue

        rule, remainder = catalog_category_for_line(line, extractor)
        if rule:
            current_category = rule_context(rule)
            current = None
            if not rule.get("parse_remainder") or not remainder:
                continue
            line = remainder

        if not current_category:
            continue

        row = parse_catalog_entry(line, extractor)
        if row:
            row.update(current_category)
            row = finish_catalog_row(row, extractor)
            rows.append(row)
            current = row
        elif current and extractor.get("append_continuation", True):
            if line_matches_any(line, extractor.get("continuation_skip_patterns", [])):
                continue
            append_value(current, description_key, line)

    skip_names = set(extractor.get("skip_names", []))
    return [row for row in rows if row.get(name_key) not in skip_names]


def name_list_rule_for_line(line, extractor):
    for rule in extractor.get("category_rules", []):
        if rule.get("text") and line == rule["text"]:
            return rule, ""
        prefix = rule.get("prefix")
        if prefix and line.startswith(prefix):
            return rule, line[len(prefix) :].strip()
    return None, None


def split_name_list(value, extractor):
    value = apply_text_fixes_to_line(value, extractor.get("name_list_text_fixes", {}))
    if extractor.get("strip_name_footnotes"):
        value = re.sub(r"(?<=\D)\d+(?:,\d+)*(?=\s*(?:,|$))", "", value)
    for marker in extractor.get("split_before_names", []):
        value = re.sub(r"\s+" + re.escape(marker), ", " + marker, value)
    separator = extractor.get("name_separator_pattern", r"\s*,\s*")
    return [
        clean_text(name)
        for name in re.split(separator, value)
        if clean_text(name)
    ]


def row_group_context(group):
    return {
        key: value
        for key, value in group.items()
        if key not in ("split_before", "name_list_text_fixes")
    }


def name_list_rows_for_group(group, value, extractor):
    group_extractor = {
        **extractor,
        "name_list_text_fixes": group.get(
            "name_list_text_fixes",
            extractor.get("name_list_text_fixes", {}),
        ),
    }
    context = row_group_context(group)
    rows = []
    for name in split_name_list(value, group_extractor):
        row = {extractor.get("name_key", "name"): name}
        row.update(context)
        rows.append(row)
    return rows


def split_line_for_groups(line, groups):
    starts = [0]
    for group in groups[1:]:
        marker = group.get("split_before")
        if not marker:
            raise ValueError("Grouped name-list rules need split_before after first group")
        index = line.find(marker)
        if index < 0:
            raise ValueError("Could not find grouped name-list split marker: " + marker)
        starts.append(index)
    values = []
    for index, start in enumerate(starts):
        end = starts[index + 1] if index + 1 < len(starts) else None
        values.append(line[start:end].strip())
    return values


def pdf_category_name_list_rows(text, extractor):
    rows = []
    pending = []

    for line in section_lines(text, extractor):
        line = apply_text_fixes_to_line(line, extractor.get("text_fixes", {}))
        if line_matches_any(line, extractor.get("skip_line_patterns", [])):
            continue

        rule, remainder = name_list_rule_for_line(line, extractor)
        if rule:
            if rule.get("groups"):
                pending.append(rule["groups"])
                continue
            context = rule_context(rule)
            if rule.get("parse_remainder") and remainder:
                rows.extend(name_list_rows_for_group(context, remainder, extractor))
            else:
                pending.append([context])
            continue

        if not pending:
            continue

        groups = pending[0]
        if len(groups) == 1:
            rows.extend(name_list_rows_for_group(groups[0], line, extractor))
            pending.pop(0)
            continue

        if any(group.get("split_before") for group in groups[1:]):
            for group, value in zip(groups, split_line_for_groups(line, groups)):
                rows.extend(name_list_rows_for_group(group, value, extractor))
            pending.pop(0)
            continue

        group = groups.pop(0)
        rows.extend(name_list_rows_for_group(group, line, extractor))
        if not groups:
            pending.pop(0)

    return rows


def pdf_wrapped_name_list_rows(text, extractor):
    rows = []
    current_context = None
    current_lines = []
    name_key = extractor.get("name_key", "name")

    def flush_current():
        if not current_context or not current_lines:
            return
        names = split_name_list(" ".join(current_lines), extractor)
        for name in names:
            row = {name_key: name}
            row.update(current_context)
            rows.append(row)

    for line in section_lines(text, extractor):
        line = apply_text_fixes_to_line(line, extractor.get("text_fixes", {}))
        if line_matches_any(line, extractor.get("skip_line_patterns", [])):
            continue

        rule, remainder = name_list_rule_for_line(line, extractor)
        if rule:
            flush_current()
            current_context = rule_context(rule)
            current_lines = []
            if rule.get("parse_remainder") and remainder:
                current_lines.append(remainder)
            continue

        if current_context:
            current_lines.append(line)

    flush_current()

    skip_names = set(extractor.get("skip_names", []))
    skip_patterns = extractor.get("skip_name_patterns", [])
    return [
        row
        for row in rows
        if row.get(name_key) not in skip_names
        and not line_matches_any(row.get(name_key, ""), skip_patterns)
    ]


def bullet_segments(line, extractor):
    bullet_pattern = extractor.get("bullet_pattern", r"-\s+")
    matches = list(re.finditer(bullet_pattern, line))
    if not matches:
        return line, []
    prefix = line[: matches[0].start()].strip()
    segments = []
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(line)
        segments.append(line[match.end() : end].strip())
    return prefix, [segment for segment in segments if segment]


def split_bullet_entry(text, extractor):
    text = clean_text(text)
    pattern = extractor.get(
        "bullet_entry_pattern",
        r"^(?P<name>.+?)(?:\s+\((?P<description>.*)\))?$",
    )
    match = re.match(pattern, text)
    if not match:
        match = re.match(
            extractor.get(
                "partial_bullet_entry_pattern",
                r"^(?P<name>.+)\s+\((?P<description>.*)$",
            ),
            text,
        )
    if not match:
        return None
    row = {
        extractor.get("name_key", "name"): match.group("name").strip(),
    }
    description = match.groupdict().get("description")
    if description:
        row[extractor.get("description_key", "description")] = description.strip()
    return row


def pdf_bullet_list_rows(text, extractor):
    rows = []
    current_category = None
    current = None
    name_key = extractor.get("name_key", "name")
    description_key = extractor.get("description_key", "description")

    for raw_line in table_section(text, extractor).splitlines():
        if extractor.get("continuation_raw_pattern"):
            may_continue = (
                re.match(extractor["continuation_raw_pattern"], raw_line) is not None
            )
        else:
            may_continue = True
        line = clean_text(raw_line)
        if not line or line_matches_any(line, extractor.get("skip_line_patterns", [])):
            continue
        line = apply_text_fixes_to_line(line, extractor.get("text_fixes", {}))
        rule, remainder = catalog_category_for_line(line, extractor)
        if rule:
            current_category = {
                key: value
                for key, value in rule.items()
                if key not in ("text", "prefix", "parse_remainder")
            }
            current = None
            if not rule.get("parse_remainder") or not remainder:
                continue
            line = remainder

        if not current_category:
            continue

        prefix, segments = bullet_segments(line, extractor)
        if segments:
            if current and prefix:
                append_value(current, description_key, prefix)
            for segment in segments:
                row = split_bullet_entry(segment, extractor)
                if not row:
                    current = None
                    continue
                row.update(current_category)
                rows.append(row)
                current = row
            continue

        if current and may_continue and extractor.get("append_continuation", True):
            if extractor.get("strip_continuation_closing_paren") and line.endswith(")"):
                line = line[:-1].strip()
            append_value(current, description_key, line)
        elif not may_continue:
            current = None

    skip_names = set(extractor.get("skip_names", []))
    skip_patterns = extractor.get("skip_name_patterns", [])
    return [
        row
        for row in rows
        if row.get(name_key) not in skip_names
        and not line_matches_any(row.get(name_key, ""), skip_patterns)
    ]


def split_quoted_entry(line, extractor):
    quote_start = re.escape(extractor.get("quote_start", "`"))
    quote_end = re.escape(extractor.get("quote_end", "'"))
    pattern = (
        r"^"
        + quote_start
        + r"(?P<name>[^`']+)"
        + quote_end
        + r"\s*(?P<description>.*)$"
    )
    match = re.match(pattern, line)
    if not match:
        return None
    name = match.group("name").strip()
    if extractor.get("repair_spaced_initial_names"):
        name = re.sub(r"^([A-Z]) ([a-z].*)$", r"\1\2", name)
    description = match.group("description").strip()
    if extractor.get("prepend_name_if_lowercase") and description:
        if description[0].islower():
            description = name + " " + description
    return {
        extractor.get("name_key", "name"): name,
        extractor.get("description_key", "description"): description,
    }


def quoted_segments(line, extractor):
    quote_start = re.escape(extractor.get("quote_start", "`"))
    quote_end = re.escape(extractor.get("quote_end", "'"))
    pattern = quote_start + r"[^`']+" + quote_end
    matches = [
        match
        for match in re.finditer(pattern, line)
        if match.start() == 0 or line[: match.start()].rstrip().endswith(".")
    ]
    if not matches:
        return line, []
    prefix = line[: matches[0].start()].strip()
    segments = []
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(line)
        segments.append(line[match.start() : end].strip())
    return prefix, [segment for segment in segments if segment]


def pdf_quoted_entry_rows(text, extractor):
    rows = []
    current_category = None
    current = None
    description_key = extractor.get("description_key", "description")
    name_key = extractor.get("name_key", "name")
    for line in section_lines(text, extractor):
        line = apply_text_fixes_to_line(line, extractor.get("text_fixes", {}))
        if line_matches_any(line, extractor.get("skip_line_patterns", [])):
            continue

        rule, remainder = catalog_category_for_line(line, extractor)
        if rule:
            current_category = {
                key: value
                for key, value in rule.items()
                if key not in ("text", "prefix", "parse_remainder")
            }
            current = None
            if not rule.get("parse_remainder") or not remainder:
                continue
            line = remainder

        if not current_category:
            continue

        prefix, segments = quoted_segments(line, extractor)
        if segments:
            if current and prefix:
                append_value(current, description_key, prefix)
            for segment in segments:
                row = split_quoted_entry(segment, extractor)
                if not row:
                    continue
                row.update(current_category)
                if (
                    extractor.get("merge_repeated_names")
                    and current
                    and row.get(name_key) == current.get(name_key)
                ):
                    append_value(current, description_key, row.get(description_key))
                    continue
                rows.append(row)
                current = row
            continue

        if current and extractor.get("append_continuation", True):
            append_value(current, description_key, line)

    skip_names = set(extractor.get("skip_names", []))
    return [row for row in rows if row.get(name_key) not in skip_names]


def apply_text_fixes_to_line(line, fixes):
    for old, new in fixes.items():
        line = line.replace(old, new)
    return line


def slice_row(line, columns, transforms=None):
    row = {}
    for column in columns:
        end = column.get("end")
        value = (
            line[column["start"] : end].strip()
            if end
            else line[column["start"] :].strip()
        )
        row[column["key"]] = clean_text(value)
    return apply_row_transforms(row, transforms or [])


def append_value(row, key, value):
    if not value:
        return
    if row.get(key):
        row[key] = row[key] + " " + value
    else:
        row[key] = value


def new_row_started(row, extractor):
    row_start_key = extractor.get("row_start_key", extractor.get("name_key", "name"))
    row_start_value = row.get(row_start_key)
    if not row_start_value:
        return False
    pattern = extractor.get("row_start_pattern")
    if pattern and not re.match(pattern, row_start_value):
        return False
    for key in extractor.get("row_start_required_keys", []):
        if not row.get(key):
            return False
    return True


def fixed_width_table_rows(text, extractor):
    columns = extractor["columns"]
    rows = []
    current = None
    started = not extractor.get("start_after_pattern")
    stop_patterns = extractor.get("stop_patterns", [])
    skip_patterns = extractor.get("skip_line_patterns", [])

    for line in table_section(text, extractor).splitlines():
        for switch in extractor.get("layout_switches", []):
            if re.search(switch["pattern"], line):
                columns = switch["columns"]
        if not started:
            if re.search(extractor["start_after_pattern"], line):
                started = True
            continue
        if stop_patterns and line_matches_any(line, stop_patterns):
            break
        if not line.strip() or line_matches_any(line, skip_patterns):
            continue

        row = slice_row(line, columns, extractor.get("row_transforms"))
        if new_row_started(row, extractor):
            if current:
                rows.append(current)
            current = row
            continue

        if current:
            for key, value in row.items():
                append_value(current, key, value)

    if current:
        rows.append(current)

    skip_names = set(extractor.get("skip_names", []))
    name_key = extractor.get("name_key", "name")
    return [row for row in rows if row.get(name_key) not in skip_names]


def group_rule_values(rule):
    control_keys = {
        "text",
        "prefix",
        "line_text",
        "line_prefix",
        "match_key",
        "heading_only",
    }
    return {key: value for key, value in rule.items() if key not in control_keys}


def group_rule_matches(row, line, extractor, rule):
    if rule.get("line_text") and line == rule["line_text"]:
        return True
    if rule.get("line_prefix") and line.startswith(rule["line_prefix"]):
        return True
    key = rule.get("match_key", extractor.get("group_key", "group"))
    value = row.get(key, "")
    if rule.get("text") and value == rule["text"]:
        return True
    if rule.get("prefix") and value.startswith(rule["prefix"]):
        return True
    return False


def group_rule_for_row(row, line, extractor):
    for rule in extractor.get("group_rules", []):
        if group_rule_matches(row, line, extractor, rule):
            return rule
    return None


def grouped_fixed_width_table_rows(text, extractor):
    columns = extractor["columns"]
    rows = []
    current = None
    current_group = None
    started = not extractor.get("start_after_pattern")
    stop_patterns = extractor.get("stop_patterns", [])
    skip_patterns = extractor.get("skip_line_patterns", [])

    for raw_line in table_section(text, extractor).splitlines():
        for switch in extractor.get("layout_switches", []):
            if re.search(switch["pattern"], raw_line):
                columns = switch["columns"]
        if not started:
            if re.search(extractor["start_after_pattern"], raw_line):
                started = True
            continue
        if stop_patterns and line_matches_any(raw_line, stop_patterns):
            break
        if not raw_line.strip() or line_matches_any(raw_line, skip_patterns):
            continue

        row = slice_row(raw_line, columns, extractor.get("row_transforms"))
        line = clean_text(raw_line)
        rule = group_rule_for_row(row, line, extractor)
        if rule:
            current_group = group_rule_values(rule)

        if new_row_started(row, extractor):
            if current:
                rows.append(current)
            current = dict(row)
            if current_group:
                current.update(current_group)
            continue

        if rule:
            continue
        if current and extractor.get("append_continuation", True):
            skip_keys = set(
                extractor.get(
                    "continuation_skip_keys",
                    [extractor.get("group_key", "group"), extractor.get("name_key", "name")],
                )
            )
            for key, value in row.items():
                if key not in skip_keys:
                    append_value(current, key, value)

    if current:
        rows.append(current)

    skip_names = set(extractor.get("skip_names", []))
    skip_patterns = extractor.get("skip_name_patterns", [])
    name_key = extractor.get("name_key", "name")
    return [
        row
        for row in rows
        if row.get(name_key) not in skip_names
        and not line_matches_any(row.get(name_key, ""), skip_patterns)
    ]


def slice_pdf_line(line, extractor):
    start = extractor.get("line_slice_start")
    end = extractor.get("line_slice_end")
    if start is None and end is None:
        return line
    return line[start:end]


def section_lines(text, extractor):
    skip_patterns = extractor.get("skip_line_patterns", [])
    lines = []
    for line in table_section(text, extractor).splitlines():
        line = slice_pdf_line(line, extractor)
        line = clean_text(line)
        if not line or line_matches_any(line, skip_patterns):
            continue
        lines.append(line)
    return lines


def numbered_blocks(text, extractor):
    start_pattern = extractor.get("block_start_pattern", r"^\d+\.$")
    blocks = []
    current = None
    for line in section_lines(text, extractor):
        if re.match(start_pattern, line):
            if current:
                blocks.append(current)
            current = [line]
        elif current:
            current.append(line)
    if current:
        blocks.append(current)
    return blocks


def split_trailing_value(text, pattern):
    match = re.match(r"^(?P<left>.+?)\s+(?P<right>" + pattern + r")$", text)
    if not match:
        return text, ""
    return match.group("left").strip(), match.group("right").strip()


def next_line_matches(lines, cursor, pattern):
    return cursor < len(lines) and fullmatch(pattern, lines[cursor])


def split_line_prefix(text, pattern):
    match = re.match(r"^(" + pattern + r")\s+(.+)$", text or "")
    if not match:
        return "", text
    return match.group(1).strip(), match.group(2).strip()


def leading_numbered_fields(block, extractor):
    zone_pattern = extractor.get("zone_pattern", r"\d(?:[- ]\d)?")
    region_pattern = extractor.get("region_pattern", r"[A-E](?:-[A-E])?")
    number_key = extractor.get("number_key", "number")
    name_key = extractor.get("name_key", "name")
    row = {number_key: block[0]}
    cursor = 1
    name_lines = []
    while cursor < len(block):
        line = block[cursor]
        name_text = clean_text(" ".join(name_lines))
        split_name, split_zone = split_trailing_value(name_text, zone_pattern)
        if split_zone and fullmatch(region_pattern, line):
            row[name_key] = split_name
            row["zones"] = split_zone
            break
        if fullmatch(zone_pattern, line):
            break
        name_lines.append(line)
        cursor += 1

    if name_key not in row:
        row[name_key] = clean_text(" ".join(name_lines))
        if next_line_matches(block, cursor, zone_pattern):
            row["zones"] = block[cursor]
            cursor += 1
        else:
            name, zones = split_trailing_value(row[name_key], zone_pattern)
            row[name_key] = name
            if zones:
                row["zones"] = zones

    tail_lines = block[cursor:]
    if next_line_matches(block, cursor, region_pattern):
        row["regions"] = block[cursor]
        tail_lines = block[cursor + 1 :]
    elif cursor < len(block):
        region, remainder = split_line_prefix(block[cursor], region_pattern)
        if region:
            row["regions"] = region
            tail_lines = [remainder] + block[cursor + 1 :]

    return row, tail_lines


def first_token_split(text, tokens):
    best = None
    for token in tokens:
        for pattern in (r"^" + re.escape(token) + r"\b", r"\b" + re.escape(token) + r"\b"):
            match = re.search(pattern, text)
            if match and (best is None or match.start() < best.start()):
                best = match
    if not best:
        return text.strip(), "", ""
    return (
        text[: best.start()].strip(),
        best.group(0).strip(),
        text[best.end() :].strip(),
    )


def parse_tail_lines(row, tail_lines, extractor):
    tail = clean_text(" ".join(tail_lines))
    tail_parser = extractor.get("tail_parser", {})
    kind = tail_parser.get("kind", "description")
    if not tail:
        return row
    if kind == "description":
        row[tail_parser.get("description_key", "description")] = tail
        return row
    if kind == "first_token_split":
        prefix, token, description = first_token_split(tail, tail_parser["tokens"])
        if prefix and tail_parser.get("prefix_key"):
            row[tail_parser["prefix_key"]] = prefix
        if token:
            row[tail_parser["token_key"]] = token
        if description:
            row[tail_parser.get("description_key", "description")] = description
        return row
    if kind == "first_line_then_description":
        prefix_key = tail_parser.get("prefix_key")
        prefix_lines = tail_lines[:1]
        description_start = 1
        for pattern in tail_parser.get("prefix_continuation_patterns", []):
            while (
                description_start < len(tail_lines)
                and fullmatch(pattern, tail_lines[description_start])
            ):
                prefix_lines.append(tail_lines[description_start])
                description_start += 1
        if prefix_key:
            row[prefix_key] = clean_text(" ".join(prefix_lines))
        description = clean_text(" ".join(tail_lines[description_start:]))
        if description:
            row[tail_parser.get("description_key", "description")] = description
        return row
    if kind == "first_line_or_token_split":
        prefix, token, description = first_token_split(tail, tail_parser["tokens"])
        if token and not prefix:
            row[tail_parser["token_key"]] = token
            if description:
                row[tail_parser.get("description_key", "description")] = description
            return row
        prefix_key = tail_parser.get("prefix_key")
        if prefix_key:
            row[prefix_key] = tail_lines[0]
        description = clean_text(" ".join(tail_lines[1:]))
        if description:
            row[tail_parser.get("description_key", "description")] = description
        return row
    raise ValueError("Unsupported numbered-block tail parser: " + kind)


def numbered_block_rows(text, extractor):
    rows = []
    for block in numbered_blocks(text, extractor):
        row, tail_lines = leading_numbered_fields(block, extractor)
        rows.append(parse_tail_lines(row, tail_lines, extractor))
    skip_names = set(extractor.get("skip_names", []))
    name_key = extractor.get("name_key", "name")
    return [row for row in rows if row.get(name_key) not in skip_names]
