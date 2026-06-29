#!/usr/bin/env python3
"""Extract a draft JSON5 reference from OSU HYG-1401 apples"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import normalized_name
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.table_tools import find_table, keyed_data_rows, table_to_dicts
from fruitfacts_extract.text_tools import join_labelled_values


SOURCE_URL = "https://ohioline.osu.edu/factsheet/hyg-1401"
CATEGORY = "Disease-resistant apple cultivars suggested for Ohio"
NAME_OVERRIDES = {
    "Pristine": ("Co-op 32", "Source table names this as Pristine"),
    "Pixie Crunch": ("Co-op 33", "Source table names this as Pixie Crunch"),
    "William's Pride": (
        "Williams' Pride",
        "Source table names this as William's Pride",
    ),
    "Goldrush": ("GoldRush", "Source table names this as Goldrush"),
}
CATEGORIES = [
    {
        "name": CATEGORY,
        "description": "Disease-resistant apple cultivars suggested by OSU for Ohio home orchards",
    }
]
REFERENCE_FIELDS = [
    ("title", "Growing Apples in the Home Orchard"),
    ("author", "Gary Y. Gao and Kass Groner"),
    ("url", SOURCE_URL),
    ("published", "Dec 22, 2025"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def row_description(row, source_note):
    parts = [join_labelled_values([("Bloom season", row["bloom_season"])])]
    parts.append(row["description"].rstrip("."))
    return append_source_note(". ".join(part for part in parts if part), source_note)


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(
        page.tables,
        ["Cultivar", "Bloom Season", "Ripening Season", "Description"],
    )
    plants = []
    for row in keyed_data_rows(table_to_dicts(table), "cultivar"):
        name, source_note = normalized_name(row["cultivar"], NAME_OVERRIDES)
        plants.append(
            plant_record(
                "Apple",
                name,
                category=CATEGORY,
                harvest_time_unparsed=row["ripening_season"],
                description=row_description(row, source_note),
            )
        )
    return plants


def emit_json5(plants):
    emit_reference(REFERENCE_FIELDS, plants, categories=CATEGORIES)


def main():
    try:
        emit_json5(extract())
    except Exception as error:
        print(f"Failed to extract OSU HYG-1401 apples: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
