#!/usr/bin/env python3
"""Extract a draft JSON5 reference from CSU GardenNotes 763"""

import sys

from fruitfacts_extract.json5_draft import q
from fruitfacts_extract.pdf_tools import clean_pdf_text, pdf_url_to_text
from fruitfacts_extract.text_tools import clean_text, split_suggested_names


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


def source_text():
    text = clean_pdf_text(pdf_url_to_text(PDF_URL))
    return clean_text(text)


def normalized_name(name):
    if name in NAME_OVERRIDES:
        return NAME_OVERRIDES[name]
    return name, None


def section_between(text, start, end):
    start_index = text.find(start)
    if start_index < 0:
        raise ValueError("Could not find section start: " + start)
    end_index = text.find(end, start_index + len(start))
    if end_index < 0:
        raise ValueError("Could not find section end: " + end)
    return text[start_index:end_index]


def suggested_names(section):
    marker = "Suggested cultivars include "
    marker_index = section.find(marker)
    if marker_index < 0:
        raise ValueError("Could not find suggested cultivar list")
    names = section[marker_index + len(marker) :].strip()
    return split_suggested_names(names)


def plants_from_category(text, category):
    section = section_between(text, category["start"], category["end"])
    plants = []
    for source_name in suggested_names(section):
        name, source_note = normalized_name(source_name)
        description = (
            "Listed by CSU GardenNotes 763 as a suggested "
            + category["name"].lower()
            + " for Colorado gardens"
        )
        if source_note:
            description += ". " + source_note
        plant = {
            "type": "Strawberry",
            "name": name,
            "category": category["name"],
            "harvest_time_unparsed": category["harvest_time_unparsed"],
            "description": description,
        }
        plants.append(plant)
    return plants


def extract():
    text = source_text()
    plants = []
    for category in CATEGORIES:
        plants.extend(plants_from_category(text, category))
    return plants


def emit_json5(plants):
    print("{")
    print('    title: "Growing Strawberries in Colorado Gardens",')
    print('    author: "David Whiting and Merrill Kingsbury",')
    print(f"    url: {q(PDF_URL)},")
    print('    published: "2018",')
    print('    reviewed: "2023",')
    print('    accessed: "Jun 2026",')
    print('    type: "state extension guide",')
    print("    needs_help: true,")
    print("    categories: [")
    for category in CATEGORIES:
        print("        {")
        print(f"            name: {q(category['name'])},")
        print(f"            description: {q(category['description'])}")
        print("        },")
    print("    ],")
    print("    plants: [")
    for plant in plants:
        print("        {")
        print(f"            type: {q(plant['type'])},")
        print(f"            name: {q(plant['name'])},")
        print(f"            category: {q(plant['category'])},")
        print(f"            harvest_time_unparsed: {q(plant['harvest_time_unparsed'])},")
        print(f"            description: {q(plant['description'])}")
        print("        },")
    print("    ]")
    print("}")
    print(f"// extracted_plants: {len(plants)}", file=sys.stderr)


def main():
    try:
        emit_json5(extract())
    except Exception as error:
        print(f"Failed to extract CSU GardenNotes 763: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
