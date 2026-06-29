#!/usr/bin/env python3
"""Extract a draft JSON5 reference from VCE 422-023 apples"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import category_records
from fruitfacts_extract.record_tools import labelled_description_from_row
from fruitfacts_extract.record_tools import plant_records_from_rows
from fruitfacts_extract.table_tools import find_table
from fruitfacts_extract.table_tools import keyed_data_rows
from fruitfacts_extract.table_tools import merge_leading_fragment_rows
from fruitfacts_extract.table_tools import table_to_dicts


SOURCE_URL = "https://pubs.ext.vt.edu/422/422-023/422-023.html"
CATEGORY = "Apple varieties recommended for Virginia"
CATEGORIES = [
    {
        "name": CATEGORY,
        "description": (
            "VCE apple varieties recommended for Virginia. Ripening dates are "
            "for Blacksburg; regions east of the Blue Ridge may be 5 to 14 "
            "days earlier. E, G, F, and P ratings mean excellent, good, fair, "
            "and poor"
        ),
    },
]
LOCATIONS = [
    {
        "name": "Virginia Tech, Blacksburg, VA",
        "latitude": 37.20248218528102,
        "longitude": -80.5631271567992,
    },
]
REFERENCE_FIELDS = [
    ("title", "Growing Apples in Virginia"),
    ("author", "Richard P. Marini"),
    ("url", SOURCE_URL),
    ("published", "Sep 3, 2025"),
    ("reviewed", "Sep 2025"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]
DESCRIPTION_LABELS = [
    ("freshz", "Fresh rating"),
    ("cookingz", "Cooking rating"),
    ("color", "Color"),
    ("taste", "Taste"),
    ("comments", "Comments"),
]


def description(row):
    return labelled_description_from_row(row, DESCRIPTION_LABELS)


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(
        page.tables,
        ["Variety", "Harvest Date", "Freshz", "Cookingz", "Color", "Taste"],
    )
    rows = keyed_data_rows(
        table_to_dicts(merge_leading_fragment_rows(table)),
        "variety",
    )
    return plant_records_from_rows(
        rows,
        "Apple",
        "variety",
        category_key=lambda row: CATEGORY,
        harvest_key="harvest_date",
        description=description,
    )


def main():
    try:
        emit_reference(
            REFERENCE_FIELDS,
            extract(),
            categories=category_records(CATEGORIES),
            locations=LOCATIONS,
        )
    except Exception as error:
        print(f"Failed to extract VCE 422-023 apples: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
