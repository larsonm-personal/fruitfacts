"""Shared text cleanup and list parsing helpers"""

import re


DEFAULT_USER_AGENT = "Mozilla/5.0 FruitFacts data helper"


def clean_text(text, collapse_whitespace=True, ascii_only=True):
    replacements = {
        "\u00a0": " ",
        "\u00b0": " degrees ",
        "\u00a9": "(c)",
        "\u00ae": "(r)",
        "\u2018": "'",
        "\u2019": "'",
        "\u201c": '"',
        "\u201d": '"',
        "\u2013": "-",
        "\u2014": "-",
        "\u2022": "*",
        "\u2026": "...",
        "\u2122": "(tm)",
        "\ufb01": "fi",
        "\ufb02": "fl",
        "\ufffd": " ",
    }
    for old, new in replacements.items():
        text = text.replace(old, new)
    if collapse_whitespace:
        text = re.sub(r"\s+", " ", text).strip()
    if ascii_only:
        text.encode("ascii")
    return text


def clean_lines(text):
    lines = []
    for line in text.splitlines():
        cleaned = clean_text(line)
        if cleaned:
            lines.append(cleaned)
    return lines


def split_suggested_names(text):
    text = text.strip().rstrip(".")
    text = text.replace(", and ", ", ")
    text = text.replace(" and ", ", ")
    return [part.strip() for part in text.split(",") if part.strip()]

