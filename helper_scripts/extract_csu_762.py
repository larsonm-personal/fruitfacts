#!/usr/bin/env python3
"""Extract a draft JSON5 reference from CSU GardenNotes 762"""

import sys

from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.pdf_tools import clean_pdf_text, pdf_url_to_text
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import category_records
from fruitfacts_extract.record_tools import normalized_name
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.text_tools import clean_text, names_after_marker, section_between


PDF_URL = "https://cmg.extension.colostate.edu/Gardennotes/762.pdf"
NAME_OVERRIDES = {
    "Chester Thornless": ("Chester", "Source text names this as Chester Thornless"),
    "Boysen (Boysenberry)": ("Boysen", "Source text names this as Boysenberry"),
    "Logan (Loganberry)": ("Logan", "Source text names this as Loganberry"),
    "Tay (Tayberry)": ("Tay", "Source text names this as Tayberry"),
}
CATEGORIES = [
    {
        "name": "Primocane-fruiting erect blackberries",
        "start": "Primocane-fruiting cultivars of erect blackberries",
        "end": "Semi-erect blackberries",
        "marker": "Suggested cultivars include ",
        "description": (
            "CSU describes primocane-fruiting erect blackberries as fruiting on "
            "new canes, making winter management easier because canes can be "
            "cut to the ground"
        ),
    },
    {
        "name": "Semi-erect blackberries",
        "start": "Semi-erect blackberries are",
        "end": "Blackberry/red raspberry hybrids",
        "marker": "Suggested cultivars include ",
        "description": (
            "CSU describes semi-erect blackberries as thornless, vigorous, "
            "trellis-requiring plants that generally yield more than trailing "
            "or erect types"
        ),
    },
    {
        "name": "Blackberry/red raspberry hybrids",
        "start": "Blackberry/red raspberry hybrids are",
        "end": "Planting and Care of Blackberries",
        "marker": "Popular cultivars include ",
        "description": (
            "CSU says blackberry-red raspberry hybrids are usually natural "
            "crosses and are generally considered a type of blackberry because "
            "the receptacle comes off with the fruit"
        ),
    },
]
REFERENCE_FIELDS = [
    ("title", "Growing Blackberries in Colorado Gardens"),
    ("author", "David Whiting and Merrill Kingsbury"),
    ("url", PDF_URL),
    ("published", "2018"),
    ("reviewed", "2023"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def source_text():
    return clean_text(clean_pdf_text(pdf_url_to_text(PDF_URL)))


def plants_from_category(text, category):
    section = section_between(text, category["start"], category["end"])
    plants = []
    for source_name in names_after_marker(section, category["marker"]):
        name, source_note = normalized_name(source_name, NAME_OVERRIDES)
        description = (
            "Listed by CSU GardenNotes 762 as a "
            + category["name"].lower()
            + " for Colorado gardens"
        )
        plants.append(
            plant_record(
                "Blackberry",
                name,
                category=category["name"],
                description=append_source_note(description, source_note),
            )
        )
    return plants


def extract():
    text = source_text()
    plants = []
    for category in CATEGORIES:
        plants.extend(plants_from_category(text, category))
    return plants


def main():
    try:
        emit_reference(REFERENCE_FIELDS, extract(), categories=category_records(CATEGORIES))
    except Exception as error:
        print(f"Failed to extract CSU GardenNotes 762: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
