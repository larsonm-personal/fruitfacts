#!/usr/bin/env python3
"""Extract a draft JSON5 reference from VCE 422-019 peaches"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import category_records
from fruitfacts_extract.record_tools import plant_records_from_rows
from fruitfacts_extract.table_tools import find_table
from fruitfacts_extract.table_tools import table_to_dicts_with_sections


SOURCE_URL = "https://pubs.ext.vt.edu/422/422-019/422-019.html"
SOURCE_NOTE = (
    "Ripening dates are for Blacksburg; lower-elevation Virginia locations may "
    "ripen 3 to 20 days earlier"
)
CATEGORIES = [
    {
        "name": "Yellow flesh peaches recommended for Virginia",
        "description": SOURCE_NOTE,
    },
    {
        "name": "White flesh peaches recommended for Virginia",
        "description": SOURCE_NOTE,
    },
    {
        "name": "Nectarines recommended for Virginia",
        "description": SOURCE_NOTE,
    },
]
LOCATIONS = [
    {
        "name": "Virginia Tech, Blacksburg, VA",
        "latitude": 37.20248218528102,
        "longitude": -80.5631271567992,
    },
]
CATEGORY_BY_SECTION = {
    "YELLOW FLESH PEACHES": "Yellow flesh peaches recommended for Virginia",
    "WHITE FLESH PEACHES": "White flesh peaches recommended for Virginia",
    "NECTARINES": "Nectarines recommended for Virginia",
}
NAME_OVERRIDES = {
    "GarnetBeauty": ("Garnet Beauty", "Source table names this as GarnetBeauty"),
    "Earnie's Choice": (
        "Ernie's Choice",
        "Source table names this as Earnie's Choice",
    ),
    "Flavor Top": ("Flavortop", "Source table names this as Flavor Top"),
}
TEXT_FIXES = {
    "ClingStone": "clingstone",
    "recom mended": "recommended",
}
REFERENCE_FIELDS = [
    ("title", "Growing Peaches and Nectarines in Virginia"),
    ("author", "Richard P. Marini"),
    ("url", SOURCE_URL),
    ("published", "Sep 3, 2025"),
    ("reviewed", "Sep 2025"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def clean_comment(text):
    for old, new in TEXT_FIXES.items():
        text = text.replace(old, new)
    return text.rstrip(".")


def expand_source_rows(rows):
    for row in rows:
        if row["variety"] != "Morton Raritan Rose":
            yield row
            continue
        note = "Source HTML combines Morton and Raritan Rose into one row"
        yield {
            **row,
            "variety": "Morton",
            "ripening_date": "July 25",
            "comments": "Small, highly colored, soft, good flavor",
            "_source_note": note,
        }
        yield {
            **row,
            "variety": "Raritan Rose",
            "ripening_date": "July 26",
            "comments": "Medium size, attractive, excellent flavor",
            "_source_note": note,
        }


def category(row):
    return CATEGORY_BY_SECTION[row["_section"]]


def plant_type(row):
    if row["_section"] == "NECTARINES":
        return "Nectarine"
    return "Peach"


def description(row):
    return append_source_note(clean_comment(row["comments"]), row.get("_source_note"))


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(page.tables, ["Variety", "Ripening Date", "Comments"])
    rows = list(
        expand_source_rows(
            table_to_dicts_with_sections(
                table,
                default_section="YELLOW FLESH PEACHES",
            )
        )
    )
    return plant_records_from_rows(
        rows,
        plant_type,
        "variety",
        category_key=category,
        harvest_key="ripening_date",
        description=description,
        name_overrides=NAME_OVERRIDES,
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
        print(f"Failed to extract VCE 422-019 peaches: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
