#!/usr/bin/env python3
"""Extract a draft JSON5 reference from OSU HYG-1401 apples"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import q
from fruitfacts_extract.table_tools import find_table, keyed_data_rows, table_to_dicts
from fruitfacts_extract.text_tools import join_labelled_values


SOURCE_URL = "https://ohioline.osu.edu/factsheet/hyg-1401"
CATEGORY = "Disease-resistant apple cultivars suggested for Ohio"
NAME_OVERRIDES = {
    "Pristine": ("Co-op 32", "Source table names this as Pristine"),
    "Pixie Crunch": ("Co-op 33", "Source table names this as Pixie Crunch"),
    "William's Pride": (
        "Williams' Pride",
        "Source table names this as William's Pride",
    ),
    "Goldrush": ("GoldRush", "Source table names this as Goldrush"),
}


def normalized_name(source_name):
    if source_name in NAME_OVERRIDES:
        return NAME_OVERRIDES[source_name]
    return source_name, None


def row_description(row, source_note):
    parts = [join_labelled_values([("Bloom season", row["bloom_season"])])]
    parts.append(row["description"].rstrip("."))
    if source_note:
        parts.append(source_note)
    return ". ".join(part for part in parts if part)


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(
        page.tables,
        ["Cultivar", "Bloom Season", "Ripening Season", "Description"],
    )
    plants = []
    for row in keyed_data_rows(table_to_dicts(table), "cultivar"):
        name, source_note = normalized_name(row["cultivar"])
        plants.append(
            {
                "type": "Apple",
                "name": name,
                "category": CATEGORY,
                "harvest_time_unparsed": row["ripening_season"],
                "description": row_description(row, source_note),
            }
        )
    return plants


def emit_json5(plants):
    print("{")
    print('    title: "Growing Apples in the Home Orchard",')
    print('    author: "Gary Y. Gao and Kass Groner",')
    print(f"    url: {q(SOURCE_URL)},")
    print('    published: "Dec 22, 2025",')
    print('    accessed: "Jun 2026",')
    print('    type: "state extension guide",')
    print("    needs_help: true,")
    print("    categories: [")
    print("        {")
    print(f"            name: {q(CATEGORY)},")
    print(
        "            description: "
        + q("Disease-resistant apple cultivars suggested by OSU for Ohio home orchards")
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
        print(f"Failed to extract OSU HYG-1401 apples: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
