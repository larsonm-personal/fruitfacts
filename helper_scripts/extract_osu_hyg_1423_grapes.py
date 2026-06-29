#!/usr/bin/env python3
"""Extract a draft JSON5 reference from OSU HYG-1423 grapes"""

import sys

from fruitfacts_extract.html_tools import fetch_html_page
from fruitfacts_extract.json5_draft import emit_reference
from fruitfacts_extract.record_tools import plant_record
from fruitfacts_extract.table_tools import find_table_with_header_row


SOURCE_URL = "https://ohioline.osu.edu/factsheet/hyg-1423"
TABLES = [
    {
        "title_contains": "Table 1",
        "category": "American cultivars suggested for Ohio",
        "description": (
            "American grape cultivars suggested by OSU for Ohio home fruit "
            "plantings"
        ),
    },
    {
        "title_contains": "Table 2",
        "category": "Seedless table grape cultivars suggested for Ohio",
        "description": (
            "Seedless table grape cultivars suggested by OSU for Ohio home "
            "fruit plantings"
        ),
    },
    {
        "title_contains": "Table 3",
        "category": "French-American hybrid cultivars suggested for Ohio",
        "description": (
            "French-American hybrid grape cultivars suggested by OSU for Ohio "
            "home fruit plantings"
        ),
    },
]
REFERENCE_FIELDS = [
    ("title", "Growing Grapes in the Home Fruit Planting"),
    ("author", "Gary Y. Gao"),
    ("url", SOURCE_URL),
    ("published", "2017"),
    ("accessed", "Jun 2026"),
    ("type", "state extension guide"),
    ("needs_help", True),
]
DISEASE_KEYS = [
    ("black_rot", "black rot"),
    ("downy_mildew", "downy mildew"),
    ("powdery_mildew", "powdery mildew"),
    ("botrytis", "botrytis"),
]


def cultivar_rows(rows):
    return [row for row in rows if row.get("cultivar") and not row["cultivar"].startswith("Note:")]


def disease_summary(row):
    if not row:
        return None
    parts = []
    for key, label in DISEASE_KEYS:
        value = row.get(key)
        if value:
            parts.append(label + " " + value)
    if not parts:
        return None
    return "Relative disease susceptibility: " + "; ".join(parts)


def description(row, disease_row):
    parts = [
        "Color: " + row["color"],
        "Winter hardiness: " + row["winter_hardiness"],
    ]
    if row.get("remarks"):
        parts.append(row["remarks"].rstrip("."))
    disease = disease_summary(disease_row)
    if disease:
        parts.append(disease)
    return ". ".join(parts)


def extract():
    page = fetch_html_page(SOURCE_URL)
    disease_table = find_table_with_header_row(
        page.tables,
        ["Cultivar", "Black Rot", "Downy Mildew", "Powdery Mildew", "Botrytis"],
        title_contains="Table 4",
    )
    diseases = {
        row["cultivar"]: row
        for row in cultivar_rows(disease_table["rows"])
    }

    categories = []
    plants = []
    for table_info in TABLES:
        table = find_table_with_header_row(
            page.tables,
            ["Cultivar", "Color", "Winter Hardiness", "Ripening Season", "Remarks"],
            title_contains=table_info["title_contains"],
        )
        categories.append(
            {
                "name": table_info["category"],
                "description": table_info["description"],
            }
        )
        for row in cultivar_rows(table["rows"]):
            name = row["cultivar"]
            plants.append(
                plant_record(
                    "Grape",
                    name,
                    category=table_info["category"],
                    harvest_time_unparsed=row["ripening_season"],
                    description=description(row, diseases.get(name)),
                )
            )
    return categories, plants


def emit_json5(categories, plants):
    emit_reference(REFERENCE_FIELDS, plants, categories=categories)


def main():
    try:
        categories, plants = extract()
        emit_json5(categories, plants)
    except Exception as error:
        print(f"Failed to extract OSU HYG-1423 grapes: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
