"""Small helpers for source rows that become draft plant records"""

import re

from fruitfacts_extract.text_tools import join_labelled_values


def normalized_name(source_name, overrides=None):
    if overrides and source_name in overrides:
        return overrides[source_name]
    return source_name, None


def strip_trailing_note_markers(source_name):
    return re.sub(r"(?<=\D)\d+(?:,\d+)*$", "", source_name).strip()


def append_source_note(description, source_note):
    if not source_note:
        return description
    if not description:
        return source_note
    return description.rstrip(".") + ". " + source_note


def plant_record(
    plant_type,
    name,
    category=None,
    harvest_time_unparsed=None,
    description=None,
):
    record = {
        "type": plant_type,
        "name": name,
    }
    if category:
        record["category"] = category
    if harvest_time_unparsed:
        record["harvest_time_unparsed"] = harvest_time_unparsed
    if description:
        record["description"] = description
    return record


def row_value(row, selector):
    if selector is None:
        return None
    if callable(selector):
        return selector(row)
    return row.get(selector)


def labelled_description_from_row(row, labels):
    return join_labelled_values(
        (label, row.get(key)) for key, label in labels if row.get(key)
    )


def plant_records_from_rows(
    rows,
    plant_type,
    name_key,
    category_key=None,
    harvest_key=None,
    description=None,
    name_overrides=None,
):
    records = []
    for row in rows:
        source_name = row_value(row, name_key)
        if not source_name:
            continue
        name, source_note = normalized_name(source_name, name_overrides)
        plant_type_value = plant_type(row) if callable(plant_type) else plant_type
        records.append(
            plant_record(
                plant_type_value,
                name,
                category=row_value(row, category_key),
                harvest_time_unparsed=row_value(row, harvest_key),
                description=append_source_note(row_value(row, description), source_note),
            )
        )
    return records


def category_records(categories):
    records = []
    for category in categories:
        record = {"name": category["name"]}
        if category.get("description"):
            record["description"] = category["description"]
        records.append(record)
    return records
