#!/usr/bin/env python3
"""Extract a draft JSON5 reference from OSU HYG-1422 blueberries"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import q
from fruitfacts_extract.table_tools import find_table, keyed_data_rows, table_to_dicts
from fruitfacts_extract.text_tools import join_labelled_values


SOURCE_URL = "https://ohioline.osu.edu/factsheet/hyg-1422"
CATEGORY = "Blueberry cultivars suggested for Ohio"


def row_description(row):
    parts = [
        join_labelled_values(
            [
                ("Yield", row["yield"]),
                ("Fruit size", row["fruit_size"]),
                ("Fruit quality", row["fruit_quality"]),
            ]
        )
    ]
    if row["remarks"]:
        parts.append(row["remarks"].rstrip("."))
    return ". ".join(part for part in parts if part)


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(
        page.tables,
        ["Cultivar", "Ripening Season", "Yield", "Fruit Size", "Fruit Quality", "Remarks"],
    )
    plants = []
    for row in keyed_data_rows(table_to_dicts(table), "cultivar"):
        plants.append(
            {
                "type": "Blueberry",
                "name": row["cultivar"],
                "category": CATEGORY,
                "harvest_time_unparsed": row["ripening_season"],
                "description": row_description(row),
            }
        )
    return plants


def emit_json5(plants):
    print("{")
    print('    title: "Growing Blueberries in the Home Garden",')
    print('    author: "Gary Y. Gao, Erik Draper, and Clifton Martin",')
    print(f"    url: {q(SOURCE_URL)},")
    print('    published: "May 15, 2026",')
    print('    accessed: "Jun 2026",')
    print('    type: "state extension guide",')
    print("    needs_help: true,")
    print("    categories: [")
    print("        {")
    print(f"            name: {q(CATEGORY)},")
    print(
        "            description: "
        + q("Blueberry cultivars suggested by OSU for Ohio home gardens")
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
        print(f"Failed to extract OSU HYG-1422 blueberries: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
