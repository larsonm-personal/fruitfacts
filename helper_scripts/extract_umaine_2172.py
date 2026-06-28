#!/usr/bin/env python3
"""Extract a draft JSON5 reference from UMaine Bulletin 2172"""

import re
import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import q


SOURCE_URL = "https://extension.umaine.edu/publications/2172e/"
NAME_OVERRIDES = {
    "Fall Gold": ("Fallgold", ["Fall Gold"]),
}


def category_from_heading(text):
    text = text.replace("raspberrie s", "rasberries")
    text = text.replace("B lack", "Black")
    text = text.replace("rasberries", "raspberries")
    match = re.match(r"(.+?)\s+\((.+)\)$", text)
    if match:
        return match.group(1), match.group(2)
    return text, None


def plant_type_for_category(category):
    if category.startswith("Blackberries"):
        return "Blackberry"
    return "Raspberry"


def harvest_hint(description):
    checks = [
        ("late August-early September", "late August-early September"),
        ("late August", "late August"),
        ("early-midseason", "early-midseason"),
        ("mid- to late-ripening", "mid to late"),
        ("mid-season ripening", "mid-season"),
        ("ripens mid-season", "mid-season"),
        ("fruit ripens mid-season", "mid-season"),
        ("late-season ripening", "late"),
        ("late ripening", "late"),
        ("ripens late", "late"),
        ("ripens early", "early"),
        ("early ripening", "early"),
        ("early-ripening", "early"),
        ("relatively early", "relatively early"),
        ("relatively late", "relatively late"),
        ("slightly before 'Heritage'", "slightly before Heritage"),
        ("slightly earlier than 'Heritage'", "slightly earlier than Heritage"),
    ]
    lower = description.lower()
    for needle, value in checks:
        if needle.lower() in lower:
            return value
    return None


def extract():
    blocks = fetch_html_page(SOURCE_URL).blocks
    plants = []
    categories = {}
    current_category = None
    started = False

    for tag, text in blocks:
        if text == "Choosing Varieties":
            started = True
            continue
        if not started:
            continue
        if text.startswith("Information in this publication"):
            break

        if tag == "h4" or text == "Blackberries, thorny (erect)":
            current_category, description = category_from_heading(text)
            categories.setdefault(current_category, [])
            if description:
                categories[current_category].append(description)
            continue

        if not current_category or tag != "p":
            continue

        if text.startswith("Note: Newer everbearing type blackberries"):
            category = "Blackberries, everbearing"
            categories[category] = [
                "Fall crop ripens too late to be practical in Maine"
            ]
            for name in ["Prime Jan", "Prime-Ark 45", "Freedom", "Traveler"]:
                plants.append(
                    {
                        "type": "Blackberry",
                        "name": name,
                        "category": category,
                        "description": (
                            "Newer everbearing blackberry. Not recommended "
                            "because fall crop ripens too late in Maine"
                        ),
                    }
                )
            continue

        match = re.match(r"([^:]{2,80}):\s+(.+)$", text)
        if not match:
            categories.setdefault(current_category, []).append(text)
            continue

        name = match.group(1).strip()
        description = match.group(2).strip()
        aka = None
        if name in NAME_OVERRIDES:
            name, aka = NAME_OVERRIDES[name]
        plant = {
            "type": plant_type_for_category(current_category),
            "name": name,
            "category": current_category,
            "description": description,
        }
        if aka:
            plant["AKA"] = aka
        harvest = harvest_hint(description)
        if harvest:
            plant["harvest_time_unparsed"] = harvest
        plants.append(plant)

    return categories, plants


def emit_json5(categories, plants):
    print("{")
    print('    title: "Raspberry and Blackberry Varieties for Maine",')
    print('    author: "David T. Handley",')
    print(f"    url: {q(SOURCE_URL)},")
    print('    published: "1995, 2009, 2020",')
    print('    accessed: "Jun 2026",')
    print('    type: "state extension guide",')
    print("    needs_help: true,")
    print("    categories: [")
    for name, description_lines in categories.items():
        description = " ".join(description_lines)
        print("        {")
        print(f"            name: {q(name)},")
        print(f"            description: {q(description)}")
        print("        },")
    print("    ],")
    print("    plants: [")
    for plant in plants:
        print("        {")
        print(f"            type: {q(plant['type'])},")
        print(f"            name: {q(plant['name'])},")
        print(f"            category: {q(plant['category'])},")
        if "AKA" in plant:
            aka_values = ", ".join(q(value) for value in plant["AKA"])
            print(f"            AKA: [{aka_values}],")
        if "harvest_time_unparsed" in plant:
            print(f"            harvest_time_unparsed: {q(plant['harvest_time_unparsed'])},")
        print(f"            description: {q(plant['description'])}")
        print("        },")
    print("    ]")
    print("}")


def main():
    try:
        categories, plants = extract()
        emit_json5(categories, plants)
    except Exception as error:
        print(f"Failed to extract UMaine 2172: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
