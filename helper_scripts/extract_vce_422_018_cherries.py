#!/usr/bin/env python3
"""Extract a draft JSON5 reference from VCE 422-018 cherries"""

import sys

from fruitfacts_extract.html_tools import blocks_between_headings
from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import category_records
from fruitfacts_extract.record_tools import plant_records_from_rows
from fruitfacts_extract.text_tools import first_sentence_containing
from fruitfacts_extract.text_tools import quoted_name_paragraph


SOURCE_URL = "https://pubs.ext.vt.edu/422/422-018/422-018.html"
CATEGORY_BY_SOURCE = {
    "Tart Cherries": "Tart cherry varieties described for Virginia",
    "Dark Sweet Cherries": "Dark sweet cherry varieties described for Virginia",
    "Light Sweet Cherries": "Light sweet cherry varieties described for Virginia",
}
CATEGORIES = [
    {
        "name": CATEGORY_BY_SOURCE["Tart Cherries"],
        "description": "Tart cherry varieties described in VCE 422-018",
    },
    {
        "name": CATEGORY_BY_SOURCE["Dark Sweet Cherries"],
        "description": "Dark sweet cherry varieties described in VCE 422-018",
    },
    {
        "name": CATEGORY_BY_SOURCE["Light Sweet Cherries"],
        "description": "Light sweet cherry varieties described in VCE 422-018",
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
    ("title", "Growing Cherries in Virginia"),
    ("author", "Richard P. Marini"),
    (
        "description",
        "VCE cherry guide with tart, dark sweet, and light sweet cherry variety descriptions, harvest timing, fruit quality, cracking, and pollination context",
    ),
    ("url", SOURCE_URL),
    ("thumbnail", "website"),
    ("published", "Sep 21, 2020"),
    ("reviewed", "Nov 2025"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def sentence_description(name, description):
    if description and description[0].islower():
        return name + " " + description
    return description


def cultivar_rows(blocks):
    source_category = "Tart Cherries"
    rows = []
    for tag, text in blocks:
        if text == "Dark Sweet Cherries":
            source_category = "Dark Sweet Cherries"
            continue
        if tag.startswith("h") and text == "Light Sweet Cherries":
            source_category = "Light Sweet Cherries"
            continue
        if tag != "p" or not text.startswith("'"):
            continue
        name, description = quoted_name_paragraph(text)
        description = sentence_description(name, description)
        rows.append(
            {
                "name": name,
                "category": CATEGORY_BY_SOURCE[source_category],
                "harvest_time_unparsed": first_sentence_containing(
                    description,
                    ["ripen"],
                ),
                "description": description,
            }
        )
    return rows


def extract():
    page = fetch_html_page(SOURCE_URL)
    blocks = blocks_between_headings(page.blocks, "Tart Cherries", "Cherry Pollination")
    return plant_records_from_rows(
        cultivar_rows(blocks),
        "Cherry",
        "name",
        category_key="category",
        harvest_key="harvest_time_unparsed",
        description="description",
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
        print(f"Failed to extract VCE 422-018 cherries: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
