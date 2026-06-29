#!/usr/bin/env python3
"""Extract a draft JSON5 reference from CSU GardenNotes 763"""

import sys

from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.pdf_tools import clean_pdf_text, pdf_url_to_text
from fruitfacts_extract.record_tools import append_source_note
from fruitfacts_extract.record_tools import category_records
from fruitfacts_extract.record_tools import normalized_name
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.text_tools import clean_text, names_after_marker, section_between


PDF_URL = "https://cmg.extension.colostate.edu/Gardennotes/763.pdf"
NAME_OVERRIDES = {
    "A.C. Wendy": ("Wendy", "Source text names this as A.C. Wendy"),
    "Bloominden Gem": (
        "Blomidon Gem",
        "Source text says Bloominden Gem, likely Blomidon Gem",
    ),
    "Carskill": ("Catskill", "Source text says Carskill, likely Catskill"),
}
CATEGORIES = [
    {
        "name": "June-bearing cultivars",
        "start": "June-bearing cultivars have",
        "end": "Ever-bearing cultivars have",
        "harvest_time_unparsed": "late June to early July along the Colorado Front Range",
        "description": (
            "CSU describes June-bearing strawberries as producing one large early "
            "summer crop with larger fruit and higher yields, but with more risk "
            "from spring temperature swings and late frosts in Colorado"
        ),
    },
    {
        "name": "Ever-bearing cultivars",
        "start": "Ever-bearing cultivars have",
        "end": "Day-neutral cultivars",
        "harvest_time_unparsed": "early summer and fall",
        "description": (
            "CSU describes ever-bearing strawberries as producing one early "
            "summer crop and a second fall crop, with smaller berries but more "
            "dependability than June-bearing types in cold Colorado climates"
        ),
    },
    {
        "name": "Day-neutral cultivars",
        "start": "Day-neutral cultivars",
        "end": "Plantings",
        "harvest_time_unparsed": "most of summer and fall",
        "description": (
            "CSU describes day-neutral strawberries as flowering through most of "
            "summer and fall in cycles, slowing during hot weather, and giving "
            "a light daily harvest with small fruit"
        ),
    },
]
REFERENCE_FIELDS = [
    ("title", "Growing Strawberries in Colorado Gardens"),
    ("author", "David Whiting and Merrill Kingsbury"),
    ("url", PDF_URL),
    ("published", "2018"),
    ("reviewed", "2023"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]


def source_text():
    text = clean_pdf_text(pdf_url_to_text(PDF_URL))
    return clean_text(text)


def plants_from_category(text, category):
    section = section_between(text, category["start"], category["end"])
    plants = []
    for source_name in names_after_marker(section, "Suggested cultivars include ", None):
        name, source_note = normalized_name(source_name, NAME_OVERRIDES)
        description = (
            "Listed by CSU GardenNotes 763 as a suggested "
            + category["name"].lower()
            + " for Colorado gardens"
        )
        plants.append(
            plant_record(
                "Strawberry",
                name,
                category=category["name"],
                harvest_time_unparsed=category["harvest_time_unparsed"],
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


def emit_json5(plants):
    emit_reference(REFERENCE_FIELDS, plants, categories=category_records(CATEGORIES))


def main():
    try:
        emit_json5(extract())
    except Exception as error:
        print(f"Failed to extract CSU GardenNotes 763: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
