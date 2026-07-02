#!/usr/bin/env python3
"""Mine cited-source strings from existing reference PDFs"""

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from fruitfacts_extract.pdf_tools import pdf_file_to_text
from fruitfacts_extract.text_tools import clean_text


FIELD_RE = re.compile(
    r"^\s*(?P<key>title|url):\s*(?P<quote>[\"'])(?P<value>.*?)(?P=quote)",
    re.MULTILINE,
)
URL_RE = re.compile(r"https?://[^\s\"'<>]+")
HEADING_RE = re.compile(
    r"^(selected\s+)?(references|literature cited|bibliography|"
    r"additional resources|resources|further reading|suggested reading|"
    r"for more information)(\s+and\s+resources)?[:]?$",
    re.IGNORECASE,
)
STOP_RE = re.compile(
    r"^(acknowledg(e)?ments|author(s)?|extension publications|"
    r"the university|issued in furtherance|copyright|disclaimer|"
    r"glossary|published by|reviewed|visit our web site)\b",
    re.IGNORECASE,
)
ENTRY_START_RE = re.compile(
    r"^(\[[0-9]+\]|[0-9]{1,3}[.)]|[-*])\s+|"
    r"^[A-Z][A-Za-z'. -]{1,35},\s+[A-Z]|"
    r"^[A-Z][A-Za-z'. -]{1,35}\s+and\s+[A-Z][A-Za-z'. -]{1,35}[,.]\s+",
)
FRUIT_RE = re.compile(
    r"\b(apple|apples|apricot|blackberr|blueberr|cherr|citrus|"
    r"cultivar|cultivars|fruit|grape|grapes|honeyberr|kiwi|nectarine|"
    r"orchard|pawpaw|peach|pear|persimmon|plum|raspberr|rootstock|"
    r"strawberr|variet|wine grape)\b",
    re.IGNORECASE,
)
SOURCE_RE = re.compile(
    r"\b(bulletin|circular|extension|fact sheet|guide|hortscience|"
    r"journal|publication|report|station|trial|university)\b",
    re.IGNORECASE,
)
SKIP_URL_RE = re.compile(
    r"//(ns\.adobe\.com|purl\.org|www\.w3\.org|iptc\.org|cipa\.jp|"
    r"www\.iec\.ch)|facebook\.com|twitter\.com|"
    r"linkedin\.com|planthardiness\.ars\.usda\.gov|"
    r"sunset\.com/garden/climate-zones|col\.st/0WMJA$|"
    r"uaf\.edu/ces$|www\.uaex\.uada\.edu$",
    re.IGNORECASE,
)
BOILERPLATE_RE = re.compile(
    r"\b(equal opportunity|affirmative action|copyright|all rights reserved|"
    r"cooperative extension work|acts of may 8 and june 30|"
    r"any products, services or organizations|contact your local extension|"
    r"download date link to item|item type authors publisher)\b",
    re.IGNORECASE,
)


def repo_root():
    return Path(__file__).resolve().parents[1]


def parse_reference_title(json5_path):
    if not json5_path.exists():
        return ""
    text = json5_path.read_text(encoding="utf-8", errors="replace")
    fields = {
        match.group("key"): match.group("value")
        for match in FIELD_RE.finditer(text)
    }
    return fields.get("title", json5_path.stem)


def pdf_paths(args):
    if args.pdf:
        return [Path(path) for path in args.pdf]
    root = args.root / "plant_database" / "references"
    return sorted(root.rglob("*.pdf"))


def source_label(pdf_path, root):
    rel = pdf_path.relative_to(root) if pdf_path.is_relative_to(root) else pdf_path
    return rel.as_posix()


def clean_url(url):
    url = clean_text(url, collapse_whitespace=False).strip()
    return url.rstrip(").,;]'\"")


def repair_wrapped_urls(text):
    text = re.sub(r"(https?://www\.)\s+", r"\1", text)
    text = re.sub(r"(https?://[A-Za-z0-9-]+)\.\s+([A-Za-z0-9.-]+)", r"\1.\2", text)
    for _ in range(3):
        text = re.sub(
            r"(https?://[^\s]+/)\s+([a-z0-9._~:/?#@!$&'()*+,;=%-]+)",
            r"\1\2",
            text,
        )
    return text


def visible_urls(text):
    text = repair_wrapped_urls(text)
    return sorted({clean_url(url) for url in URL_RE.findall(text) if not skip_url(url)})


def binary_urls(pdf_path):
    data = pdf_path.read_bytes()
    urls = set()
    for raw in re.findall(rb"https?://[^\s<>\"{}|\\^`\[\]]+", data):
        url = clean_url(raw.decode("latin1", errors="ignore").replace("\x00", ""))
        if url and not skip_url(url):
            urls.add(url)
    return sorted(urls)


def skip_url(url):
    return SKIP_URL_RE.search(url) is not None


def cleaned_pdf_lines(text):
    lines = []
    for line in text.splitlines():
        cleaned = clean_text(line)
        lines.append(cleaned)
    return lines


def heading_indexes(lines):
    indexes = []
    for index, line in enumerate(lines):
        if line and HEADING_RE.match(line):
            indexes.append(index)
    return indexes


def citation_sections(lines, max_lines):
    sections = []
    for start in heading_indexes(lines):
        collected = []
        for line in lines[start + 1 : start + max_lines + 1]:
            if line and STOP_RE.match(line):
                break
            collected.append(line)
        if collected:
            sections.append(collected)
    return sections


def should_skip_line(line):
    if not line or BOILERPLATE_RE.search(line):
        return True
    return len(line) < 8 and not URL_RE.search(line)


def starts_entry(line):
    return ENTRY_START_RE.search(line) is not None


def citation_blocks(lines):
    blocks = []
    current = []
    for line in lines:
        if should_skip_line(line):
            if current:
                blocks.append(" ".join(current))
                current = []
            continue
        if current and starts_entry(line):
            blocks.append(" ".join(current))
            current = [line]
        else:
            current.append(line)
        if sum(len(part) for part in current) > 900:
            blocks.append(" ".join(current))
            current = []
    if current:
        blocks.append(" ".join(current))
    return [clean_text(block) for block in blocks if useful_candidate(block)]


def context_blocks_for_urls(lines, radius=2):
    blocks = []
    for index, line in enumerate(lines):
        if not visible_urls(line):
            continue
        start = max(0, index - radius)
        end = min(len(lines), index + radius + 1)
        context = " ".join(line for line in lines[start:end] if line)
        if useful_candidate(context):
            blocks.append(clean_text(context))
    return blocks


def useful_candidate(text):
    if BOILERPLATE_RE.search(text):
        return False
    if len(clean_text(text)) < 20:
        return False
    return bool(FRUIT_RE.search(text) or SOURCE_RE.search(text) or visible_urls(text))


def score_candidate(text, urls, kind):
    score = 0
    if kind == "reference_section":
        score += 30
    if urls:
        score += 20 if kind == "embedded_url" else 35
        if any(".pdf" in urlparse_path(url).lower() for url in urls):
            score += 15
    has_fruit = FRUIT_RE.search(text) or any(FRUIT_RE.search(url) for url in urls)
    has_source = SOURCE_RE.search(text) or any(SOURCE_RE.search(url) for url in urls)
    if has_fruit:
        score += 30
    if has_source:
        score += 15
    if re.search(r"\b(19|20)[0-9]{2}\b", text):
        score += 10
    if kind == "embedded_url" and not has_fruit and not has_source:
        score = 0
    return score


def urlparse_path(url):
    match = re.match(r"https?://[^/]+(?P<path>/.*)?", url)
    return match.group("path") or "" if match else url


def candidate_string(pdf_path, title, kind, text, urls, score, root):
    pieces = [
        f"score={score}",
        f"kind={kind}",
        f"source_pdf={source_label(pdf_path, root)}",
    ]
    if title:
        pieces.append(f"source_title={title}")
    if urls:
        pieces.append("urls=" + ", ".join(urls))
    pieces.append("candidate=" + clean_text(text))
    return " | ".join(pieces)


def scan_pdf(pdf_path, root, max_section_lines):
    try:
        text = pdf_file_to_text(pdf_path, layout=False)
    except Exception as error:
        return [
            candidate_string(
                pdf_path,
                parse_reference_title(pdf_path.with_suffix(".json5")),
                "pdf_text_error",
                str(error),
                [],
                0,
                root,
            )
        ]
    title = parse_reference_title(pdf_path.with_suffix(".json5"))
    lines = cleaned_pdf_lines(text)
    found = []
    for section in citation_sections(lines, max_section_lines):
        for block in citation_blocks(section):
            urls = visible_urls(block)
            score = score_candidate(block, urls, "reference_section")
            found.append(
                candidate_string(
                    pdf_path, title, "reference_section", block, urls, score, root
                )
            )
    for block in context_blocks_for_urls(lines):
        urls = visible_urls(block)
        score = score_candidate(block, urls, "visible_url_context")
        found.append(candidate_string(pdf_path, title, "visible_url_context", block, urls, score, root))
    for url in binary_urls(pdf_path):
        score = score_candidate(url, [url], "embedded_url")
        found.append(candidate_string(pdf_path, title, "embedded_url", url, [url], score, root))
    return found


def normalized_candidate(candidate):
    normalized = re.sub(r"source_pdf=[^|]+\|\s*", "", candidate)
    normalized = re.sub(r"source_title=[^|]+\|\s*", "", normalized)
    normalized = re.sub(r"score=[0-9]+\|\s*", "", normalized)
    return clean_text(normalized).lower()


def candidate_score(candidate):
    match = re.match(r"score=([0-9]+)", candidate)
    return int(match.group(1)) if match else 0


def candidate_source_pdf(candidate):
    match = re.search(r"source_pdf=([^|]+)", candidate)
    if not match:
        raise ValueError("Candidate string is missing source_pdf: " + candidate)
    return Path(match.group(1).strip())


def candidate_reference_path(candidate, root):
    pdf_path = candidate_source_pdf(candidate)
    if not pdf_path.is_absolute():
        pdf_path = root / pdf_path
    json5_path = pdf_path.with_suffix(".json5")
    if not json5_path.exists():
        raise ValueError("Could not find reference JSON5 for " + str(pdf_path))
    return json5_path


def load_candidate_strings(path):
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, list) or not all(isinstance(item, str) for item in data):
        raise ValueError("Input file must be a JSON array of strings")
    return data


def grouped_candidates(candidates, root):
    grouped = {}
    for candidate in candidates:
        path = candidate_reference_path(candidate, root)
        grouped.setdefault(path, []).append(candidate)
    return grouped


def remove_top_level_array_field(text, field_name):
    match = re.search(rf"(?m)^    {re.escape(field_name)}:\s*\[", text)
    if not match:
        return text
    index = match.end() - 1
    depth = 0
    in_string = False
    escaped = False
    while index < len(text):
        char = text[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
        else:
            if char == '"':
                in_string = True
            elif char == "[":
                depth += 1
            elif char == "]":
                depth -= 1
                if depth == 0:
                    index += 1
                    while index < len(text) and text[index] in " \t":
                        index += 1
                    if index < len(text) and text[index] == ",":
                        index += 1
                    if index < len(text) and text[index] in "\r\n":
                        if text[index : index + 2] == "\r\n":
                            index += 2
                        else:
                            index += 1
                    return text[: match.start()] + text[index:]
        index += 1
    raise ValueError("Unclosed top-level array field: " + field_name)


def citation_field_block(citations, field_name):
    lines = [f"    {field_name}: ["]
    for citation in citations:
        lines.append("        " + json.dumps(citation, ensure_ascii=True) + ",")
    lines.append("    ],")
    return "\n".join(lines) + "\n"


def insert_top_level_field(text, field_name, citations):
    text = remove_top_level_array_field(text, field_name)
    block = citation_field_block(citations, field_name)
    for marker in ("\n    locations:", "\n    categories:", "\n    plants:"):
        index = text.find(marker)
        if index >= 0:
            return text[: index + 1] + block + text[index + 1 :]
    close_index = text.rfind("\n}")
    if close_index < 0:
        raise ValueError("Could not find reference object end")
    return text[: close_index + 1] + block + text[close_index + 1 :]


def write_reference_json5_citations(candidates, root, field_name):
    changed = []
    for path, citations in sorted(grouped_candidates(candidates, root).items()):
        raw = path.read_bytes()
        newline = "\r\n" if b"\r\n" in raw else "\n"
        text = raw.decode("utf-8").replace("\r\n", "\n")
        updated = insert_top_level_field(text, field_name, citations)
        if updated != text:
            path.write_bytes(updated.replace("\n", newline).encode("utf-8"))
            changed.append(path)
    return changed


def scan(args):
    root = args.root
    candidates = []
    for index, path in enumerate(pdf_paths(args)):
        if args.max_pdfs and index >= args.max_pdfs:
            break
        candidates.extend(scan_pdf(path, root, args.max_section_lines))
    unique = {}
    for candidate in candidates:
        if candidate_score(candidate) < args.min_score:
            continue
        key = normalized_candidate(candidate)
        if key not in unique or candidate_score(candidate) > candidate_score(unique[key]):
            unique[key] = candidate
    ranked = sorted(unique.values(), key=lambda item: (-candidate_score(item), item))
    if args.limit:
        ranked = ranked[: args.limit]
    return ranked


def self_test():
    lines = cleaned_pdf_lines(
        """
        References
        1. Smith, J. 2020. Apple cultivar trial report. University Extension.
        Available at https://example.edu/apple-trial.pdf
        2. Boilerplate equal opportunity statement
        """
    )
    blocks = citation_blocks(citation_sections(lines, 20)[0])
    assert len(blocks) == 1
    assert "Apple cultivar trial" in blocks[0]
    assert visible_urls(blocks[0]) == ["https://example.edu/apple-trial.pdf"]
    assert visible_urls("See https://catalog. extension.oregonstate.edu/ec1309") == [
        "https://catalog.extension.oregonstate.edu/ec1309"
    ]
    inserted = insert_top_level_field(
        '{\n    title: "Example",\n    locations: [],\n    plants: [],\n}\n',
        "found_citations",
        ["source_pdf=example.pdf | candidate=Apple trial"],
    )
    assert "found_citations" in inserted
    replaced = insert_top_level_field(
        inserted,
        "found_citations",
        ["source_pdf=example.pdf | candidate=Pear trial"],
    )
    assert replaced.count("found_citations") == 1
    assert "Pear trial" in replaced and "Apple trial" not in replaced


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Mine candidate source strings from local reference PDFs"
    )
    parser.add_argument("--root", type=Path, default=repo_root(), help="Repository root")
    parser.add_argument("--pdf", type=Path, action="append", help="PDF path to scan")
    parser.add_argument("--limit", type=int, default=80, help="Maximum candidates to print")
    parser.add_argument("--max-pdfs", type=int, help="Maximum PDFs to scan")
    parser.add_argument("--min-score", type=int, default=45, help="Minimum candidate score")
    parser.add_argument("--output", type=Path, help="Write JSON array to this path")
    parser.add_argument("--input", type=Path, help="Read an existing JSON array of strings")
    parser.add_argument(
        "--write-reference-json5",
        action="store_true",
        help="Group candidate strings by source_pdf and write them into reference JSON5 files",
    )
    parser.add_argument(
        "--field-name",
        default="found_citations",
        help="Reference JSON5 field used by --write-reference-json5",
    )
    parser.add_argument(
        "--max-section-lines",
        type=int,
        default=90,
        help="Maximum lines after a citation heading to inspect",
    )
    parser.add_argument("--self-test", action="store_true", help="Run scanner self-test")
    args = parser.parse_args(argv)

    if args.self_test:
        self_test()
        print("self-test passed")
        return 0

    candidates = load_candidate_strings(args.input) if args.input else scan(args)
    if args.write_reference_json5:
        changed = write_reference_json5_citations(candidates, args.root, args.field_name)
        for path in changed:
            print(path.relative_to(args.root).as_posix())
        return 0

    output = json.dumps(candidates, indent=4) + "\n"
    if args.output:
        args.output.write_text(output, encoding="utf-8")
    else:
        print(output, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
