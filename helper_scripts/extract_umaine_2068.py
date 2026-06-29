#!/usr/bin/env python3
"""Extract a draft JSON5 reference from UMaine Bulletin 2068"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.table_tools import fill_leading_group_cells
from fruitfacts_extract.table_tools import find_table
from fruitfacts_extract.table_tools import keyed_data_rows
from fruitfacts_extract.table_tools import table_to_dicts
from fruitfacts_extract.text_tools import join_labelled_values


SOURCE_URL = "https://extension.umaine.edu/publications/2068e/"
CATEGORIES = [
    {
        "name": "Yellow flesh",
        "description": "Yellow-fleshed peach varieties listed by UMaine",
    },
    {
        "name": "White Flesh",
        "description": "White-fleshed peach varieties listed by UMaine",
    },
    {
        "name": "Flat (Donut)",
        "description": "Flat or donut peach varieties listed by UMaine",
    },
]
REFERENCE_FIELDS = [
    ("title", "Growing Peaches in Maine"),
    ("author", "Renae Moran"),
    ("url", SOURCE_URL),
    ("published", "2024"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def source_name_note(source_name):
    if not source_name.endswith("*"):
        return source_name, None
    return source_name.rstrip("*"), "Evaluated at the University of Maine Highmoor Farm"


def row_description(row, source_note):
    description = join_labelled_values(
        [
            ("Flower bud cold hardiness", row["flower_bud_cold_hardiness"]),
            ("Leaf curl resistance", row["leaf_curl_resistance"]),
            ("Fruit size", row["fruit_size"]),
            ("Fruit quality", row["fruit_quality"]),
        ]
    )
    return append_source_note(description, source_note)


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(
        page.tables,
        [
            "Type",
            "Variety",
            "Flower bud cold hardiness",
            "Ripening date",
            "Leaf curl resistance",
            "Fruit size",
            "Fruit quality",
        ],
    )
    rows = keyed_data_rows(table_to_dicts(fill_leading_group_cells(table)), "variety")
    plants = []
    for row in rows:
        name, source_note = source_name_note(row["variety"])
        plants.append(
            plant_record(
                "Peach",
                name,
                category=row["type"],
                harvest_time_unparsed=row["ripening_date"],
                description=row_description(row, source_note),
            )
        )
    return plants


def main():
    try:
        emit_reference(REFERENCE_FIELDS, extract(), categories=CATEGORIES)
    except Exception as error:
        print(f"Failed to extract UMaine 2068: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
