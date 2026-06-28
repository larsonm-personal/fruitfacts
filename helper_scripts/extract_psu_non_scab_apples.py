#!/usr/bin/env python3
"""Extract a draft JSON5 reference from Penn State non-scab apple table"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import q
from fruitfacts_extract.table_tools import find_table, table_to_dicts


SOURCE_URL = (
    "https://extension.psu.edu/"
    "home-orchards-table-4-2-non-scab-resistant-apple-varieties/"
)
NAME_OVERRIDES = {
    "Zestar!": ("Minnewashta", "Source table names this as Zestar!"),
    "Ginger Gold": ("Mountain Cove", "Source table names this as Ginger Gold"),
    "Blondee": ("McLaughlin Gala", "Source table names this as Blondee"),
    "Cameo": ("Caudle", "Source table names this as Cameo"),
    "SunCrisp": ("NJ 55", "Source table names this as SunCrisp"),
}


def normalized_name(source_name):
    if source_name in NAME_OVERRIDES:
        return NAME_OVERRIDES[source_name]
    return source_name, None


def extract():
    page = fetch_html_page(SOURCE_URL)
    table = find_table(page.tables, ["Variety", "Characteristics", "Ripening Period"])
    plants = []
    for row in table_to_dicts(table):
        source_name = row["variety"]
        name, source_note = normalized_name(source_name)
        description = row["characteristics"].rstrip(".")
        if source_note:
            description += ". " + source_note
        plants.append(
            {
                "type": "Apple",
                "name": name,
                "harvest_time_unparsed": row["ripening_period"],
                "description": description,
            }
        )
    return plants


def emit_json5(plants):
    print("{")
    print('    title: "Penn State Non-Scab-Resistant Apple Varieties",')
    print('    author: "Donald Seifrit",')
    print(f"    url: {q(SOURCE_URL)},")
    print('    reviewed: "2026",')
    print('    accessed: "Jun 2026",')
    print('    type: "state extension guide",')
    print("    needs_help: true,")
    print("    plants: [")
    for plant in plants:
        print("        {")
        print(f"            type: {q(plant['type'])},")
        print(f"            name: {q(plant['name'])},")
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
        print(f"Failed to extract Penn State non-scab apples: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
