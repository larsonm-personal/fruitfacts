"""Helpers for table-like text extracted from PDFs"""

import re

from fruitfacts_extract.text_tools import clean_text
from fruitfacts_extract.text_tools import section_between


def table_section(text, extractor):
    section = text
    if extractor.get("section_start"):
        if extractor.get("section_end"):
            section = section_between(
                text,
                extractor["section_start"],
                extractor["section_end"],
            )
        else:
            start_index = text.find(extractor["section_start"])
            if start_index < 0:
                raise ValueError("Could not find section start: " + extractor["section_start"])
            section = text[start_index:]
    return section


def line_matches_any(line, patterns):
    return any(re.search(pattern, line) for pattern in patterns)


def fullmatch(pattern, value):
    return re.fullmatch(pattern, value or "") is not None


def split_suffix(row, transform):
    value = row.get(transform["source_key"], "")
    for suffix in sorted(transform["suffixes"], key=len, reverse=True):
        pattern = r"^(.*?)\s+" + re.escape(suffix) + r"$"
        match = re.match(pattern, value)
        if not match:
            continue
        row[transform["left_key"]] = match.group(1).strip()
        row[transform["right_key"]] = suffix
        return row
    row[transform["left_key"]] = value
    row[transform["right_key"]] = ""
    return row


def split_on_token(row, transform):
    value = row.get(transform["source_key"], "")
    for token in sorted(transform["tokens"], key=len, reverse=True):
        pattern = r"^(.*?)\s+" + re.escape(token) + r"(?:\s+(.*))?$"
        match = re.match(pattern, value)
        if not match:
            continue
        row[transform["left_key"]] = match.group(1).strip()
        row[transform["right_key"]] = token
        remainder = (match.group(2) or "").strip()
        if remainder and transform.get("remainder_key"):
            target_key = transform["remainder_key"]
            existing = row.get(target_key, "")
            joiner = transform.get("remainder_joiner", " ")
            row[target_key] = (remainder + joiner + existing).strip()
        return row
    row[transform["left_key"]] = value
    row[transform["right_key"]] = ""
    return row


def split_prefix_token(row, transform):
    value = row.get(transform["source_key"], "")
    for token in sorted(transform["tokens"], key=len, reverse=True):
        pattern = r"^" + re.escape(token) + r"(?:\s+(.*))?$"
        match = re.match(pattern, value)
        if not match:
            continue
        row[transform["left_key"]] = token
        row[transform["right_key"]] = (match.group(1) or "").strip()
        return row
    row[transform["left_key"]] = ""
    row[transform["right_key"]] = value
    return row


def apply_row_transforms(row, transforms):
    for transform in transforms:
        if transform["kind"] == "split_suffix":
            row = split_suffix(row, transform)
        elif transform["kind"] == "split_on_token":
            row = split_on_token(row, transform)
        elif transform["kind"] == "split_prefix_token":
            row = split_prefix_token(row, transform)
        else:
            raise ValueError("Unsupported PDF table row transform: " + transform["kind"])
    return row


def slice_row(line, columns, transforms=None):
    row = {}
    for column in columns:
        end = column.get("end")
        value = (
            line[column["start"] : end].strip()
            if end
            else line[column["start"] :].strip()
        )
        row[column["key"]] = clean_text(value)
    return apply_row_transforms(row, transforms or [])


def append_value(row, key, value):
    if not value:
        return
    if row.get(key):
        row[key] = row[key] + " " + value
    else:
        row[key] = value


def new_row_started(row, extractor):
    row_start_key = extractor.get("row_start_key", extractor.get("name_key", "name"))
    row_start_value = row.get(row_start_key)
    if not row_start_value:
        return False
    pattern = extractor.get("row_start_pattern")
    if pattern and not re.match(pattern, row_start_value):
        return False
    for key in extractor.get("row_start_required_keys", []):
        if not row.get(key):
            return False
    return True


def fixed_width_table_rows(text, extractor):
    columns = extractor["columns"]
    rows = []
    current = None
    started = not extractor.get("start_after_pattern")
    stop_patterns = extractor.get("stop_patterns", [])
    skip_patterns = extractor.get("skip_line_patterns", [])

    for line in table_section(text, extractor).splitlines():
        for switch in extractor.get("layout_switches", []):
            if re.search(switch["pattern"], line):
                columns = switch["columns"]
        if not started:
            if re.search(extractor["start_after_pattern"], line):
                started = True
            continue
        if stop_patterns and line_matches_any(line, stop_patterns):
            break
        if not line.strip() or line_matches_any(line, skip_patterns):
            continue

        row = slice_row(line, columns, extractor.get("row_transforms"))
        if new_row_started(row, extractor):
            if current:
                rows.append(current)
            current = row
            continue

        if current:
            for key, value in row.items():
                append_value(current, key, value)

    if current:
        rows.append(current)

    skip_names = set(extractor.get("skip_names", []))
    name_key = extractor.get("name_key", "name")
    return [row for row in rows if row.get(name_key) not in skip_names]


def section_lines(text, extractor):
    skip_patterns = extractor.get("skip_line_patterns", [])
    lines = []
    for line in table_section(text, extractor).splitlines():
        line = clean_text(line)
        if not line or line_matches_any(line, skip_patterns):
            continue
        lines.append(line)
    return lines


def numbered_blocks(text, extractor):
    start_pattern = extractor.get("block_start_pattern", r"^\d+\.$")
    blocks = []
    current = None
    for line in section_lines(text, extractor):
        if re.match(start_pattern, line):
            if current:
                blocks.append(current)
            current = [line]
        elif current:
            current.append(line)
    if current:
        blocks.append(current)
    return blocks


def split_trailing_value(text, pattern):
    match = re.match(r"^(?P<left>.+?)\s+(?P<right>" + pattern + r")$", text)
    if not match:
        return text, ""
    return match.group("left").strip(), match.group("right").strip()


def next_line_matches(lines, cursor, pattern):
    return cursor < len(lines) and fullmatch(pattern, lines[cursor])


def split_line_prefix(text, pattern):
    match = re.match(r"^(" + pattern + r")\s+(.+)$", text or "")
    if not match:
        return "", text
    return match.group(1).strip(), match.group(2).strip()


def leading_numbered_fields(block, extractor):
    zone_pattern = extractor.get("zone_pattern", r"\d(?:[- ]\d)?")
    region_pattern = extractor.get("region_pattern", r"[A-E](?:-[A-E])?")
    number_key = extractor.get("number_key", "number")
    name_key = extractor.get("name_key", "name")
    row = {number_key: block[0]}
    cursor = 1
    name_lines = []
    while cursor < len(block):
        line = block[cursor]
        name_text = clean_text(" ".join(name_lines))
        split_name, split_zone = split_trailing_value(name_text, zone_pattern)
        if split_zone and fullmatch(region_pattern, line):
            row[name_key] = split_name
            row["zones"] = split_zone
            break
        if fullmatch(zone_pattern, line):
            break
        name_lines.append(line)
        cursor += 1

    if name_key not in row:
        row[name_key] = clean_text(" ".join(name_lines))
        if next_line_matches(block, cursor, zone_pattern):
            row["zones"] = block[cursor]
            cursor += 1
        else:
            name, zones = split_trailing_value(row[name_key], zone_pattern)
            row[name_key] = name
            if zones:
                row["zones"] = zones

    tail_lines = block[cursor:]
    if next_line_matches(block, cursor, region_pattern):
        row["regions"] = block[cursor]
        tail_lines = block[cursor + 1 :]
    elif cursor < len(block):
        region, remainder = split_line_prefix(block[cursor], region_pattern)
        if region:
            row["regions"] = region
            tail_lines = [remainder] + block[cursor + 1 :]

    return row, tail_lines


def first_token_split(text, tokens):
    best = None
    for token in tokens:
        for pattern in (r"^" + re.escape(token) + r"\b", r"\b" + re.escape(token) + r"\b"):
            match = re.search(pattern, text)
            if match and (best is None or match.start() < best.start()):
                best = match
    if not best:
        return text.strip(), "", ""
    return (
        text[: best.start()].strip(),
        best.group(0).strip(),
        text[best.end() :].strip(),
    )


def parse_tail_lines(row, tail_lines, extractor):
    tail = clean_text(" ".join(tail_lines))
    tail_parser = extractor.get("tail_parser", {})
    kind = tail_parser.get("kind", "description")
    if not tail:
        return row
    if kind == "description":
        row[tail_parser.get("description_key", "description")] = tail
        return row
    if kind == "first_token_split":
        prefix, token, description = first_token_split(tail, tail_parser["tokens"])
        if prefix and tail_parser.get("prefix_key"):
            row[tail_parser["prefix_key"]] = prefix
        if token:
            row[tail_parser["token_key"]] = token
        if description:
            row[tail_parser.get("description_key", "description")] = description
        return row
    if kind == "first_line_then_description":
        prefix_key = tail_parser.get("prefix_key")
        prefix_lines = tail_lines[:1]
        description_start = 1
        for pattern in tail_parser.get("prefix_continuation_patterns", []):
            while (
                description_start < len(tail_lines)
                and fullmatch(pattern, tail_lines[description_start])
            ):
                prefix_lines.append(tail_lines[description_start])
                description_start += 1
        if prefix_key:
            row[prefix_key] = clean_text(" ".join(prefix_lines))
        description = clean_text(" ".join(tail_lines[description_start:]))
        if description:
            row[tail_parser.get("description_key", "description")] = description
        return row
    if kind == "first_line_or_token_split":
        prefix, token, description = first_token_split(tail, tail_parser["tokens"])
        if token and not prefix:
            row[tail_parser["token_key"]] = token
            if description:
                row[tail_parser.get("description_key", "description")] = description
            return row
        prefix_key = tail_parser.get("prefix_key")
        if prefix_key:
            row[prefix_key] = tail_lines[0]
        description = clean_text(" ".join(tail_lines[1:]))
        if description:
            row[tail_parser.get("description_key", "description")] = description
        return row
    raise ValueError("Unsupported numbered-block tail parser: " + kind)


def numbered_block_rows(text, extractor):
    rows = []
    for block in numbered_blocks(text, extractor):
        row, tail_lines = leading_numbered_fields(block, extractor)
        rows.append(parse_tail_lines(row, tail_lines, extractor))
    skip_names = set(extractor.get("skip_names", []))
    name_key = extractor.get("name_key", "name")
    return [row for row in rows if row.get(name_key) not in skip_names]
