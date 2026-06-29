#!/usr/bin/env python3
"""Extract a draft JSON5 reference from UMaine Bulletin 2253"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import normalized_name
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.table_tools import find_table, keyed_data_rows, table_to_dicts
from fruitfacts_extract.text_tools import join_labelled_values


SOURCE_URL = "https://extension.umaine.edu/publications/2253e/"
CATEGORY = "Highbush Blueberry Varieties for Northern New England"
NAME_OVERRIDES = {
    "Blue Gold": ("Bluegold", "Source table names this as Blue Gold"),
}
CATEGORIES = [
    {
        "name": CATEGORY,
        "description": "Highbush blueberry varieties listed by UMaine for Northern New England",
    }
]
REFERENCE_FIELDS = [
    ("title", "Growing Highbush Blueberries"),
    ("author", "David T. Handley"),
    ("url", SOURCE_URL),
    ("published", "1992, 2008"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(
        page.tables,
        ["Variety", "Plant Characteristics", "Fruit Qualities", "Ripening Season"],
    )
    plants = []
    for row in keyed_data_rows(table_to_dicts(table), "variety"):
        source_name = row["variety"]
        name, source_note = normalized_name(source_name, NAME_OVERRIDES)
        description = join_labelled_values(
            [
                ("Plant characteristics", row["plant_characteristics"]),
                ("Fruit qualities", row["fruit_qualities"]),
            ]
        )
        plants.append(
            plant_record(
                "Blueberry",
                name,
                category=CATEGORY,
                harvest_time_unparsed=row["ripening_season"],
                description=append_source_note(description, source_note),
            )
        )
    return plants


def emit_json5(plants):
    emit_reference(REFERENCE_FIELDS, plants, categories=CATEGORIES)


def main():
    try:
        emit_json5(extract())
    except Exception as error:
        print(f"Failed to extract UMaine 2253: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
