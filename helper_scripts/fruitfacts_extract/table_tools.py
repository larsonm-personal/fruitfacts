"""Helpers for simple extracted HTML tables"""

import re


def header_key(text):
    key = text.lower()
    key = re.sub(r"[^a-z0-9]+", "_", key)
    return key.strip("_")


def table_to_dicts(table):
    if not table:
        return []
    headers = [header_key(value) for value in table[0]]
    rows = []
    for source_row in table[1:]:
        row = {}
        for index, header in enumerate(headers):
            row[header] = source_row[index] if index < len(source_row) else ""
        rows.append(row)
    return rows


def keyed_data_rows(rows, key, skip_prefixes=("Note:",)):
    data = []
    for row in rows:
        value = row.get(key)
        if not value:
            continue
        if any(value.startswith(prefix) for prefix in skip_prefixes):
            continue
        data.append(row)
    return data


def fill_leading_group_cells(table):
    if not table:
        return []
    headers = table[0]
    expected_length = len(headers)
    group_value = None
    aligned = [headers]
    for row in table[1:]:
        if len(row) == expected_length:
            group_value = row[0]
            aligned.append(row)
        elif len(row) == expected_length - 1 and group_value:
            aligned.append([group_value] + row)
        else:
            raise ValueError("Could not align grouped table row: " + repr(row))
    return aligned


def table_from_header_row(table, required_headers):
    wanted = {header_key(value) for value in required_headers}
    for index, row in enumerate(table):
        headers = [header_key(value) for value in row]
        if wanted.issubset(set(headers)):
            title = " ".join(" ".join(title_row).strip() for title_row in table[:index])
            return {
                "title": title.strip(),
                "headers": row,
                "rows": table_to_dicts(table[index:]),
            }
    raise ValueError("Could not find header row: " + ", ".join(required_headers))


def find_table(tables, required_headers):
    wanted = {header_key(value) for value in required_headers}
    for table in tables:
        if not table:
            continue
        headers = {header_key(value) for value in table[0]}
        if wanted.issubset(headers):
            return table
    raise ValueError("Could not find table with headers: " + ", ".join(required_headers))


def find_table_with_header_row(tables, required_headers, title_contains=None):
    for table in tables:
        if title_contains:
            title_text = " ".join(" ".join(row) for row in table[:2])
            if title_contains not in title_text:
                continue
        try:
            return table_from_header_row(table, required_headers)
        except ValueError:
            pass
    raise ValueError("Could not find table with header row: " + ", ".join(required_headers))
