"""Shared text cleanup and list parsing helpers"""

import re
import unicodedata


DEFAULT_USER_AGENT = "Mozilla/5.0 FruitFacts data helper"


def clean_text(text, collapse_whitespace=True, ascii_only=True):
    replacements = {
        "\u00a0": " ",
        "\u00bc": "1/4",
        "\u00bd": "1/2",
        "\u00be": "3/4",
        "\u00b0": " degrees ",
        "\u00ba": " degrees ",
        "\u00a9": "(c)",
        "\u00ae": "(r)",
        "\u00ad": "",
        "\u2018": "'",
        "\u2019": "'",
        "\u201c": '"',
        "\u201d": '"',
        "\u2010": "-",
        "\u2011": "-",
        "\u2013": "-",
        "\u2014": "-",
        "\u2022": "*",
        "\u200b": "",
        "\u2026": "...",
        "\u2212": "-",
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
        text = unicodedata.normalize("NFKD", text)
        text = text.encode("ascii", "ignore").decode("ascii")
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


def join_labelled_values(parts):
    sentences = []
    for label, value in parts:
        if not value:
            continue
        cleaned_label = clean_text(label).rstrip(":")
        cleaned_value = clean_text(value).rstrip(".")
        sentences.append(cleaned_label + ": " + cleaned_value)
    return ". ".join(sentences)


def split_sentences(text):
    return [part.strip() for part in re.split(r"(?<=[.!?])\s+", text) if part.strip()]


def first_sentence_containing(text, terms):
    normalized_terms = [term.lower() for term in terms]
    for sentence in split_sentences(text):
        lowered = sentence.lower()
        if any(term in lowered for term in normalized_terms):
            return sentence
    return None


def quoted_name_paragraph(text):
    match = re.match(r"^'((?:[^']|'(?=[A-Za-z]))+)'\s*(?:-\s*)?(.*)$", text)
    if not match:
        raise ValueError("Could not parse quoted-name paragraph: " + text)
    return match.group(1), match.group(2).strip()


def quoted_names(text):
    matches = re.findall(r"'((?:[^']|'(?=[A-Za-z]))+)'(?=$|[\s,.;:)])", text)
    return [match.strip() for match in matches if match.strip()]


def section_between(text, start, end):
    start_index = text.find(start)
    if start_index < 0:
        raise ValueError("Could not find section start: " + start)
    end_index = text.find(end, start_index + len(start))
    if end_index < 0:
        raise ValueError("Could not find section end: " + end)
    return text[start_index:end_index]


def names_after_marker(text, marker, end_marker="."):
    marker_index = text.find(marker)
    if marker_index < 0:
        raise ValueError("Could not find list marker: " + marker)
    names_text = text[marker_index + len(marker) :].strip()
    if end_marker:
        names_text = names_text.split(end_marker, 1)[0]
    return split_suggested_names(names_text)
