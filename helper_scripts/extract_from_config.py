#!/usr/bin/env python3
"""Extract a draft JSON5 reference from a parser config"""

import json
import sys
from pathlib import Path

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import category_records
from fruitfacts_extract.record_tools import labelled_description_from_row
from fruitfacts_extract.record_tools import normalized_name
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.record_tools import strip_trailing_note_markers
from fruitfacts_extract.table_tools import find_table
from fruitfacts_extract.table_tools import find_table_with_header_row
from fruitfacts_extract.table_tools import keyed_data_rows
from fruitfacts_extract.table_tools import table_to_dicts
from fruitfacts_extract.text_tools import first_sentence_containing


def load_config(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def pairs(data):
    return [(item[0], item[1]) for item in data]


def name_overrides(config):
    return {
        source: (target["name"], target.get("note"))
        for source, target in config.get("name_overrides", {}).items()
    }


def source_page(config):
    source = config["source"]
    if source["kind"] != "html":
        raise ValueError("Unsupported source kind: " + source["kind"])
    return fetch_html_page(source["url"])


def table_rows(page, extractor):
    if extractor.get("header_row"):
        table = find_table_with_header_row(
            page.tables,
            extractor["required_headers"],
            title_contains=extractor.get("title_contains"),
        )
        rows = table["rows"]
    else:
        table = find_table(page.tables, extractor["required_headers"])
        rows = table_to_dicts(table)
    return keyed_data_rows(
        rows,
        extractor["name_key"],
        skip_prefixes=tuple(extractor.get("skip_prefixes", ["Note:"])),
    )


def description(row, extractor):
    description_value = labelled_description_from_row(
        row,
        pairs(extractor.get("description_labels", [])),
    )
    if extractor.get("description_suffix"):
        description_value = append_source_note(
            description_value,
            extractor["description_suffix"],
        )
    return description_value


def harvest_time(row, extractor):
    if extractor.get("harvest_key"):
        return row.get(extractor["harvest_key"])
    if extractor.get("harvest_from_key"):
        return first_sentence_containing(
            row.get(extractor["harvest_from_key"], ""),
            extractor.get("harvest_terms", ["ripen", "matur"]),
        )
    return None


def plants_from_extractor(page, extractor, overrides):
    plants = []
    for row in table_rows(page, extractor):
        source_name = row[extractor["name_key"]]
        if extractor.get("strip_name_footnotes"):
            source_name = strip_trailing_note_markers(source_name)
        name, source_note = normalized_name(source_name, overrides)
        plants.append(
            plant_record(
                extractor["plant_type"],
                name,
                category=extractor.get("category"),
                harvest_time_unparsed=harvest_time(row, extractor),
                description=append_source_note(description(row, extractor), source_note),
            )
        )
    return plants


def extract(config):
    page = source_page(config)
    overrides = name_overrides(config)
    plants = []
    for extractor in config["extractors"]:
        if extractor["kind"] != "html_table":
            raise ValueError("Unsupported extractor kind: " + extractor["kind"])
        plants.extend(plants_from_extractor(page, extractor, overrides))
    return plants


def main(argv):
    if len(argv) != 2:
        print("Usage: extract_from_config.py CONFIG.json", file=sys.stderr)
        return 2
    try:
        config = load_config(argv[1])
        emit_reference(
            pairs(config["reference_fields"]),
            extract(config),
            categories=category_records(config.get("categories", [])),
            locations=config.get("locations"),
        )
    except Exception as error:
        print(f"Failed to extract from config: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
