#!/usr/bin/env python3
"""Extract a draft JSON5 reference from CSU GardenNotes 764"""

import sys

from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.pdf_tools import clean_pdf_text
from fruitfacts_extract.pdf_tools import pdf_url_to_text
from fruitfacts_extract.record_tools import category_records
from fruitfacts_extract.record_tools import plant_records_from_rows
from fruitfacts_extract.text_tools import clean_text
from fruitfacts_extract.text_tools import names_after_marker
from fruitfacts_extract.text_tools import section_between


PDF_URL = "https://cmg.extension.colostate.edu/Gardennotes/764.pdf"
NAME_OVERRIDES = {
    "Niagra": ("Niagara", "Source text says Niagra, likely Niagara"),
}
CATEGORIES = [
    {
        "name": "Table grape cultivars popular in Colorado gardens",
        "marker": "Popular cultivars include ",
        "end_marker": ". Juice and jelly",
        "description": (
            "CSU describes table grapes as used for fresh eating and says most "
            "popular cultivars are seedless"
        ),
    },
    {
        "name": "Juice and jelly grape cultivars popular in Colorado gardens",
        "marker": "Juice and jelly grapes whose popular cultivars include ",
        "end_marker": ". Wine grapes",
        "description": "CSU lists these as popular juice and jelly grape cultivars",
    },
]
LOCATIONS = [
    {
        "name": "Colorado State University Extension, Fort Collins, Colorado",
        "latitude": 40.5748,
        "longitude": -105.0808,
    },
]
REFERENCE_FIELDS = [
    ("title", "Growing Grapes in Colorado Gardens"),
    ("author", "David Whiting and Merrill Kingsbury"),
    (
        "description",
        "Colorado Master Gardener GardenNotes grape guide with popular table, juice, and jelly grape cultivars and home vineyard management context",
    ),
    ("url", PDF_URL),
    ("thumbnail", "pdf"),
    ("reviewed", "Mar 2023"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def source_text():
    return clean_text(clean_pdf_text(pdf_url_to_text(PDF_URL)))


def cultivar_rows(text):
    section = section_between(text, "Types of Grapes", "Types of Cultivars")
    rows = []
    for category in CATEGORIES:
        for name in names_after_marker(section, category["marker"], category["end_marker"]):
            rows.append(
                {
                    "name": name,
                    "category": category["name"],
                    "description": "Listed by CSU GardenNotes 764 as a popular grape cultivar for Colorado gardens",
                }
            )
    return rows


def extract():
    return plant_records_from_rows(
        cultivar_rows(source_text()),
        "Grape",
        "name",
        category_key="category",
        description="description",
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
        print(f"Failed to extract CSU GardenNotes 764: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
