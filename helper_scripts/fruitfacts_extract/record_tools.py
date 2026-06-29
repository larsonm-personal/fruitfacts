"""Small helpers for source rows that become draft plant records"""


def normalized_name(source_name, overrides=None):
    if overrides and source_name in overrides:
        return overrides[source_name]
    return source_name, None


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


def category_records(categories):
    records = []
    for category in categories:
        record = {"name": category["name"]}
        if category.get("description"):
            record["description"] = category["description"]
        records.append(record)
    return records
