#!/usr/bin/env python3
"""Extract a draft JSON5 reference from UMaine Bulletin 2253"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import labelled_description_from_row
from fruitfacts_extract.record_tools import plant_records_from_rows
from fruitfacts_extract.table_tools import find_table, keyed_data_rows, table_to_dicts


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
    return plant_records_from_rows(
        keyed_data_rows(table_to_dicts(table), "variety"),
        "Blueberry",
        "variety",
        category_key=lambda row: CATEGORY,
        harvest_key="ripening_season",
        description=lambda row: labelled_description_from_row(
            row,
            [
                ("plant_characteristics", "Plant characteristics"),
                ("fruit_qualities", "Fruit qualities"),
            ],
        ),
        name_overrides=NAME_OVERRIDES,
    )


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
