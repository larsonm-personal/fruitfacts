"""JSON5 draft emission helpers"""

import json
import sys


def q(value):
    return json.dumps(value, ensure_ascii=True)


def emit_field(name, value, indent=1, trailing_comma=True):
    comma = "," if trailing_comma else ""
    prefix = "    " * indent
    if isinstance(value, bool):
        value_text = "true" if value else "false"
    else:
        value_text = q(value)
    print(f"{prefix}{name}: {value_text}{comma}")


def emit_object(data, indent=2, trailing_comma=True):
    prefix = "    " * indent
    print(prefix + "{")
    items = list(data.items())
    for index, (key, value) in enumerate(items):
        emit_field(key, value, indent + 1, trailing_comma=index < len(items) - 1)
    print(prefix + "}" + ("," if trailing_comma else ""))


def emit_object_list(name, values, indent=1):
    prefix = "    " * indent
    print(prefix + name + ": [")
    for value in values:
        emit_object(value, indent + 1)
    print(prefix + "],")


def emit_reference(fields, plants, categories=None, locations=None):
    print("{")
    for name, value in fields:
        emit_field(name, value)
    if locations is not None:
        emit_object_list("locations", locations)
    if categories:
        emit_object_list("categories", categories)
    print("    plants: [")
    for plant in plants:
        emit_object(plant, 2)
    print("    ]")
    print("}")
    print(f"// extracted_plants: {len(plants)}", file=sys.stderr)
