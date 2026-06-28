#!/usr/bin/env python3
"""Extract a draft JSON5 reference from UMaine Bulletin 2184"""

import re
import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import q


SOURCE_URL = "https://extension.umaine.edu/publications/2184e/"
SEASON_HEADINGS = {
    "Early Season",
    "Early-Midseason",
    "Midseason",
    "Mid-Late Season",
    "Late Season",
    "Day-Neutral",
}


def table_summary(tables):
    summaries = {}
    for table in tables:
        if not table or table[0][:4] != [
            "Variety",
            "Ripening Time",
            "Pest Resistance",
            "Comments",
        ]:
            continue
        for row in table[1:]:
            if len(row) < 4:
                continue
            summaries[row[0]] = {
                "ripening": row[1],
                "pest_resistance": row[2],
                "comments": row[3],
            }
    return summaries


def description_from_following(blocks, index):
    if index + 1 >= len(blocks):
        return None
    next_tag, next_text = blocks[index + 1]
    if next_tag != "p" or re.match(r"[^:]{2,80}:\s*", next_text):
        return None
    return next_text


def harvest_hint(description, table_row):
    lower = description.lower()
    checks = [
        ("late summer-fall", "late summer-fall"),
        ("very early ripening", "very early"),
        ("early ripening", "early"),
        ("early, late summer-fall ripening", "early and late summer-fall"),
        ("late ripening", "late"),
    ]
    for needle, value in checks:
        if needle in lower:
            return value
    if table_row and table_row.get("ripening"):
        return table_row["ripening"]
    return None


def extra_description_bits(table_row):
    bits = []
    if not table_row:
        return bits
    if table_row.get("pest_resistance"):
        bits.append("Summary table lists pest resistance: " + table_row["pest_resistance"])
    if table_row.get("comments"):
        bits.append("Summary table note: " + table_row["comments"])
    return bits


def extract():
    page = fetch_html_page(SOURCE_URL)
    summaries = table_summary(page.tables)
    plants = []
    current_category = None
    pending_skip_index = None

    for index, (tag, text) in enumerate(page.blocks):
        if index == pending_skip_index:
            continue

        if tag == "h3" and text in SEASON_HEADINGS:
            current_category = text
            continue
        if not current_category or tag != "p":
            continue
        if text.startswith("Information in this publication"):
            break
        if text.startswith("VIDEO:"):
            break

        match = re.match(r"([^:]{2,80}):(?:\s+(.+))?$", text)
        if not match:
            continue

        name = match.group(1).strip()
        description = (match.group(2) or "").strip()
        if not description:
            description = description_from_following(page.blocks, index)
            if description:
                pending_skip_index = index + 1
        if not description:
            continue

        table_row = summaries.get(name)
        bits = [description]
        bits.extend(extra_description_bits(table_row))
        plant = {
            "type": "Strawberry",
            "name": name,
            "category": current_category,
            "description": " ".join(bits),
        }
        harvest = harvest_hint(description, table_row)
        if harvest:
            plant["harvest_time_unparsed"] = harvest
        plants.append(plant)

    return summaries, plants


def emit_json5(summaries, plants):
    print("{")
    print('    title: "Strawberry Varieties for Maine",')
    print('    author: "David T. Handley",')
    print(f"    url: {q(SOURCE_URL)},")
    print('    published: "2011",')
    print('    reviewed: "2024",')
    print('    accessed: "Jun 2026",')
    print('    type: "state extension guide",')
    print("    needs_help: true,")
    print("    plants: [")
    for plant in plants:
        print("        {")
        print(f"            type: {q(plant['type'])},")
        print(f"            name: {q(plant['name'])},")
        print(f"            category: {q(plant['category'])},")
        if "harvest_time_unparsed" in plant:
            print(f"            harvest_time_unparsed: {q(plant['harvest_time_unparsed'])},")
        print(f"            description: {q(plant['description'])}")
        print("        },")
    print("    ]")
    print("}")
    print(f"// extracted_plants: {len(plants)}", file=sys.stderr)
    print(f"// table_rows: {len(summaries)}", file=sys.stderr)


def main():
    try:
        summaries, plants = extract()
        emit_json5(summaries, plants)
    except Exception as error:
        print(f"Failed to extract UMaine 2184: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
