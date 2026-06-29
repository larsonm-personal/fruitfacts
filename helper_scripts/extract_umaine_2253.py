#!/usr/bin/env python3
"""Extract a draft JSON5 reference from UMaine Bulletin 2253"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import q
from fruitfacts_extract.table_tools import find_table, table_to_dicts
from fruitfacts_extract.text_tools import join_labelled_values


SOURCE_URL = "https://extension.umaine.edu/publications/2253e/"
CATEGORY = "Highbush Blueberry Varieties for Northern New England"
NAME_OVERRIDES = {
    "Blue Gold": ("Bluegold", "Source table names this as Blue Gold"),
}


def normalized_name(source_name):
    if source_name in NAME_OVERRIDES:
        return NAME_OVERRIDES[source_name]
    return source_name, None


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(
        page.tables,
        ["Variety", "Plant Characteristics", "Fruit Qualities", "Ripening Season"],
    )
    plants = []
    for row in table_to_dicts(table):
        source_name = row["variety"]
        name, source_note = normalized_name(source_name)
        description = join_labelled_values(
            [
                ("Plant characteristics", row["plant_characteristics"]),
                ("Fruit qualities", row["fruit_qualities"]),
            ]
        )
        if source_note:
            description += ". " + source_note
        plants.append(
            {
                "type": "Blueberry",
                "name": name,
                "category": CATEGORY,
                "harvest_time_unparsed": row["ripening_season"],
                "description": description,
            }
        )
    return plants


def emit_json5(plants):
    print("{")
    print('    title: "Growing Highbush Blueberries",')
    print('    author: "David T. Handley",')
    print(f"    url: {q(SOURCE_URL)},")
    print('    published: "1992, 2008",')
    print('    accessed: "Jun 2026",')
    print('    type: "state extension guide",')
    print("    needs_help: true,")
    print("    categories: [")
    print("        {")
    print(f"            name: {q(CATEGORY)},")
    print(
        "            description: "
        + q("Highbush blueberry varieties listed by UMaine for Northern New England")
    )
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
        print(f"Failed to extract UMaine 2253: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
