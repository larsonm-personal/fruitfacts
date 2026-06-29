#!/usr/bin/env python3
"""Extract a draft JSON5 reference from Penn State non-scab apple table"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import normalized_name
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.table_tools import find_table, keyed_data_rows, table_to_dicts


SOURCE_URL = (
    "https://extension.psu.edu/"
    "home-orchards-table-4-2-non-scab-resistant-apple-varieties/"
)
NAME_OVERRIDES = {
    "Zestar!": ("Minnewashta", "Source table names this as Zestar!"),
    "Ginger Gold": ("Mountain Cove", "Source table names this as Ginger Gold"),
    "Blondee": ("McLaughlin Gala", "Source table names this as Blondee"),
    "Cameo": ("Caudle", "Source table names this as Cameo"),
    "SunCrisp": ("NJ 55", "Source table names this as SunCrisp"),
}
REFERENCE_FIELDS = [
    ("title", "Penn State Non-Scab-Resistant Apple Varieties"),
    ("author", "Donald Seifrit"),
    ("url", SOURCE_URL),
    ("reviewed", "2026"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(page.tables, ["Variety", "Characteristics", "Ripening Period"])
    plants = []
    for row in keyed_data_rows(table_to_dicts(table), "variety"):
        source_name = row["variety"]
        name, source_note = normalized_name(source_name, NAME_OVERRIDES)
        plants.append(
            plant_record(
                "Apple",
                name,
                harvest_time_unparsed=row["ripening_period"],
                description=append_source_note(row["characteristics"], source_note),
            )
        )
    return plants


def emit_json5(plants):
    emit_reference(REFERENCE_FIELDS, plants)


def main():
    try:
        emit_json5(extract())
    except Exception as error:
        print(f"Failed to extract Penn State non-scab apples: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
