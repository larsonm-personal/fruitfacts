#!/usr/bin/env python3
"""Extract a draft JSON5 reference from UMaine Bulletin 2184"""

from html.parser import HTMLParser
import json
import re
import sys
from urllib.request import Request, urlopen


SOURCE_URL = "https://extension.umaine.edu/publications/2184e/"
USER_AGENT = "Mozilla/5.0 FruitFacts data helper"
SEASON_HEADINGS = {
    "Early Season",
    "Early-Midseason",
    "Midseason",
    "Mid-Late Season",
    "Late Season",
    "Day-Neutral",
}


class PageParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.blocks = []
        self.tables = []
        self.current = None
        self.skip_depth = 0
        self.in_table = False
        self.current_row = None
        self.current_cell = None

    def handle_starttag(self, tag, attrs):
        if tag in ("script", "style", "noscript"):
            self.skip_depth += 1
            return
        if self.skip_depth:
            return

        if tag == "table":
            self.in_table = True
            self.tables.append([])
            return
        if self.in_table:
            if tag == "tr":
                self.current_row = []
            elif tag in ("th", "td") and self.current_row is not None:
                self.current_cell = []
            elif tag == "br" and self.current_cell is not None:
                self.current_cell.append(" ")
            return

        if tag in ("h1", "h2", "h3", "h4", "p", "li"):
            self.current = [tag, []]
        elif self.current and tag == "br":
            self.current[1].append(" ")

    def handle_endtag(self, tag):
        if tag in ("script", "style", "noscript"):
            self.skip_depth = max(0, self.skip_depth - 1)
            return
        if self.skip_depth:
            return

        if self.in_table:
            if tag in ("th", "td") and self.current_cell is not None:
                text = clean_text("".join(self.current_cell))
                self.current_row.append(text)
                self.current_cell = None
            elif tag == "tr" and self.current_row is not None:
                if any(self.current_row):
                    self.tables[-1].append(self.current_row)
                self.current_row = None
            elif tag == "table":
                self.in_table = False
            return

        if self.current and tag == self.current[0]:
            text = clean_text("".join(self.current[1]))
            if text:
                self.blocks.append((tag, text))
            self.current = None

    def handle_data(self, data):
        if self.skip_depth:
            return
        if self.current_cell is not None:
            self.current_cell.append(data)
        elif self.current:
            self.current[1].append(data)


def clean_text(text):
    replacements = {
        "\u00a0": " ",
        "\u00a9": "(c)",
        "\u00ae": "(r)",
        "\u2018": "'",
        "\u2019": "'",
        "\u201c": '"',
        "\u201d": '"',
        "\u2013": "-",
        "\u2014": "-",
        "\u2026": "...",
        "\u2122": "(tm)",
    }
    for old, new in replacements.items():
        text = text.replace(old, new)
    text = re.sub(r"\s+", " ", text).strip()
    text.encode("ascii")
    return text


def fetch_page():
    request = Request(SOURCE_URL, headers={"User-Agent": USER_AGENT})
    html = urlopen(request).read().decode("utf-8", "replace")
    parser = PageParser()
    parser.feed(html)
    return parser.blocks, parser.tables


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
    blocks, tables = fetch_page()
    summaries = table_summary(tables)
    plants = []
    current_category = None
    pending_skip_index = None

    for index, (tag, text) in enumerate(blocks):
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
            description = description_from_following(blocks, index)
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


def q(value):
    return json.dumps(value, ensure_ascii=True)


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
