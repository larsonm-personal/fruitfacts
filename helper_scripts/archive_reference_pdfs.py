#!/usr/bin/env python3
"""Find and download PDF companions for reference JSON5 files"""

import argparse
import http.client
import re
import sys
import time
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urljoin, urlparse, urlunparse
from urllib.request import Request, urlopen

from fruitfacts_extract.text_tools import DEFAULT_USER_AGENT, clean_text


REFERENCE_FIELD_RE = re.compile(
    r"^\s*(?P<key>title|url|thumbnail):\s*(?P<quote>[\"'])(?P<value>.*?)(?P=quote)",
    re.MULTILINE,
)
URL_RE = re.compile(r"https?://[^\s\"'<>]+")
PRINT_PDF_RE = re.compile(
    r"\b(print\s+friendly\s+pdf|download\s+pdf|pdf\s+download|pdf\s+version|"
    r"view\s+pdf|full\s+text\s+pdf|publication\s+pdf|printable\s+pdf)\b",
    re.IGNORECASE,
)
PDF_WORD_RE = re.compile(r"\bpdf\b", re.IGNORECASE)
DOWNLOAD_RE = re.compile(r"\bdownload\b", re.IGNORECASE)
PDF_HREF_RE = re.compile(r"(?:\.pdf(?:$|[?#])|/pdf(?:$|[/?#]))", re.IGNORECASE)
SKIP_PDF_RE = re.compile(
    r"\b(application form|registration form|poster|flyer|agenda|newsletter|privacy|accessibility)\b",
    re.IGNORECASE,
)


@dataclass
class Reference:
    path: Path
    title: str
    url: str
    thumbnail: str


@dataclass
class PdfCandidate:
    url: str
    score: int
    reason: str
    text: str = ""


class PdfLinkParser(HTMLParser):
    def __init__(self, base_url):
        super().__init__()
        self.base_url = base_url
        self.links = []
        self.current_link = None

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "a":
            self.current_link = [attrs.get("href", ""), []]
            return
        if tag == "link":
            href = attrs.get("href")
            if href and "pdf" in " ".join(
                attrs.get(key, "") for key in ("type", "rel", "title")
            ).lower():
                self.links.append(
                    (
                        clean_text(attrs.get("title", "PDF alternate")),
                        urljoin(self.base_url, href),
                        "link tag",
                    )
                )
            return
        if tag == "meta":
            name = (attrs.get("name") or attrs.get("property") or "").lower()
            content = attrs.get("content")
            if content and "pdf" in name:
                self.links.append(
                    (
                        clean_text(name),
                        urljoin(self.base_url, content),
                        "meta tag",
                    )
                )

    def handle_endtag(self, tag):
        if tag == "a" and self.current_link is not None:
            href = self.current_link[0]
            text = clean_text("".join(self.current_link[1]))
            if href:
                self.links.append((text, urljoin(self.base_url, href), "link"))
            self.current_link = None

    def handle_data(self, data):
        if self.current_link is not None:
            self.current_link[1].append(data)


def repo_root():
    return Path(__file__).resolve().parents[1]


def request_bytes(url, timeout=45, retries=2):
    headers = {
        "User-Agent": DEFAULT_USER_AGENT,
        "Accept": "application/pdf,text/html,*/*",
    }
    last_error = None
    for attempt in range(retries + 1):
        try:
            request = Request(url, headers=headers)
            with urlopen(request, timeout=timeout) as response:
                return response.read(), response.headers, response.geturl()
        except (HTTPError, URLError, TimeoutError, http.client.RemoteDisconnected) as error:
            last_error = error
            if attempt < retries:
                time.sleep(0.75 * (attempt + 1))
    raise last_error


def parse_reference(path):
    fields = {}
    text = path.read_text(encoding="utf-8", errors="replace")
    for match in REFERENCE_FIELD_RE.finditer(text):
        fields.setdefault(match.group("key"), match.group("value"))
    url_field = fields.get("url", "")
    urls = URL_RE.findall(url_field)
    url = urls[0] if urls else url_field
    if not url.startswith(("http://", "https://")):
        return None
    return Reference(
        path=path,
        title=fields.get("title", path.stem),
        url=url,
        thumbnail=fields.get("thumbnail", ""),
    )


def references_from_args(args):
    if args.reference:
        return [
            reference
            for path in args.reference
            if (reference := parse_reference(path))
        ]
    root = args.repo_root / "plant_database" / "references"
    return [
        reference
        for path in sorted(root.rglob("*.json5"))
        if (reference := parse_reference(path))
    ]


def has_local_pdf_asset(reference):
    pdf_path = reference.path.with_suffix(".pdf")
    return pdf_path.exists() or Path(str(pdf_path) + ".dvc").exists()


def is_pdf_url(url):
    return PDF_HREF_RE.search(urlparse(url).path) is not None


def same_domain(left, right):
    return urlparse(left).netloc.lower() == urlparse(right).netloc.lower()


def score_pdf_link(source_url, text, href, origin):
    lowered = " ".join([text, href]).lower()
    if SKIP_PDF_RE.search(lowered):
        return None
    score = 0
    reasons = []
    href_is_pdf = is_pdf_url(href)
    if href_is_pdf:
        score += 55
        reasons.append("pdf href")
    if PRINT_PDF_RE.search(text):
        score += 90
        reasons.append("print/download pdf text")
    elif href_is_pdf and DOWNLOAD_RE.search(text):
        score += 45
        reasons.append("download text")
    elif PDF_WORD_RE.search(text):
        score += 20
        reasons.append("pdf text")
    if origin in ("link tag", "meta tag"):
        score += 55
        reasons.append(origin)
    if same_domain(source_url, href):
        score += 10
        reasons.append("same domain")
    if not reasons:
        return None
    return PdfCandidate(
        url=href,
        score=score,
        reason=", ".join(reasons),
        text=text,
    )


def parse_pdf_links(source_url, html):
    parser = PdfLinkParser(source_url)
    parser.feed(html)
    candidates = []
    seen = set()
    for text, href, origin in parser.links:
        without_fragment = urlunparse(urlparse(href)._replace(fragment=""))
        if without_fragment in seen:
            continue
        seen.add(without_fragment)
        candidate = score_pdf_link(source_url, text, without_fragment, origin)
        if candidate:
            candidates.append(candidate)
    return candidates


def derived_pdf_candidates(url):
    parsed = urlparse(url)
    path = parsed.path
    candidates = []
    if path.endswith("/index.html"):
        pdf_path = path[: -len("/index.html")] + ".pdf"
        candidates.append((pdf_path, "derived from index.html"))
    if path.endswith(".html"):
        candidates.append((path[: -len(".html")] + ".pdf", "derived from html"))
    return [
        PdfCandidate(
            url=urlunparse(parsed._replace(path=pdf_path, query="", fragment="")),
            score=70,
            reason=reason,
        )
        for pdf_path, reason in candidates
    ]


def discover_candidates(reference):
    candidates = []
    if is_pdf_url(reference.url):
        candidates.append(PdfCandidate(reference.url, 200, "reference url is pdf"))
        return candidates

    html_bytes, _, final_url = request_bytes(reference.url)
    html = html_bytes.decode("utf-8", "replace")
    candidates.extend(parse_pdf_links(final_url, html))
    candidates.extend(derived_pdf_candidates(final_url))

    deduped = {}
    for candidate in candidates:
        existing = deduped.get(candidate.url)
        if not existing or candidate.score > existing.score:
            deduped[candidate.url] = candidate
    return sorted(deduped.values(), key=lambda item: item.score, reverse=True)


def is_pdf_bytes(data, headers):
    content_type = headers.get("Content-Type", "").lower()
    return data.lstrip().startswith(b"%PDF-") or "application/pdf" in content_type


def is_html_bytes(data, headers):
    content_type = headers.get("Content-Type", "").lower()
    stripped = data.lstrip()[:200].lower()
    return (
        "text/html" in content_type
        or stripped.startswith(b"<!doctype html")
        or stripped.startswith(b"<html")
    )


def accepted_candidates(candidates, threshold):
    return [candidate for candidate in candidates if candidate.score >= threshold]


def download_candidate_pdf(candidate, threshold, verbose=False, seen=None, depth=0):
    if seen is None:
        seen = set()
    if candidate.url in seen:
        return None
    seen.add(candidate.url)

    try:
        data, headers, final_url = request_bytes(candidate.url, timeout=90)
    except Exception as error:
        if verbose:
            print(f"download failed for {candidate.url}: {error}")
        return None

    if is_pdf_bytes(data, headers):
        return data, final_url, candidate.reason

    if depth < 1 and not is_pdf_url(candidate.url) and is_html_bytes(data, headers):
        html = data.decode("utf-8", "replace")
        child_candidates = accepted_candidates(
            parse_pdf_links(final_url, html) + derived_pdf_candidates(final_url),
            threshold,
        )
        for child in child_candidates:
            result = download_candidate_pdf(
                child,
                threshold,
                verbose=verbose,
                seen=seen,
                depth=depth + 1,
            )
            if result:
                pdf_data, pdf_url, child_reason = result
                return (
                    pdf_data,
                    pdf_url,
                    candidate.reason + "; landing page " + child_reason,
                )

    if verbose:
        print(f"rejected non-PDF response for {candidate.url}")
    return None


def archive_reference(reference, args):
    pdf_path = reference.path.with_suffix(".pdf")
    if has_local_pdf_asset(reference) and not args.force:
        return "skipped", f"already has {pdf_path.name}"

    try:
        candidates = accepted_candidates(discover_candidates(reference), args.min_score)
    except Exception as error:
        return "error", f"could not inspect source: {error}"

    if not candidates:
        return "missing", "no high-confidence PDF candidate"

    first = candidates[0]
    if args.dry_run:
        return "found", f"{first.url} [{first.reason}]"

    for candidate in candidates:
        result = download_candidate_pdf(
            candidate,
            args.min_score,
            verbose=args.verbose,
        )
        if not result:
            continue
        data, final_url, reason = result
        pdf_path.write_bytes(data)
        return "downloaded", f"{pdf_path} from {final_url} [{reason}]"

    return "error", "candidates did not return PDF bytes"


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Discover and download PDF companions for reference JSON5 files",
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=repo_root(),
        help="Repository root",
    )
    parser.add_argument(
        "--reference",
        type=Path,
        action="append",
        help="Reference JSON5 file to inspect, may be repeated",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Find PDF candidates without downloading files",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Redownload even when a sibling PDF or PDF DVC pointer exists",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Return a nonzero exit code if any source has an inspection error",
    )
    parser.add_argument(
        "--min-score",
        type=int,
        default=80,
        help="Minimum confidence score for a PDF candidate",
    )
    parser.add_argument(
        "--max-downloads",
        type=int,
        help="Stop after this many new PDFs have been downloaded",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print skipped references and rejected candidates",
    )
    return parser.parse_args(argv)


def main(argv=None):
    args = parse_args(argv)
    args.repo_root = args.repo_root.resolve()
    downloaded = 0
    found = 0
    errors = 0

    for reference in references_from_args(args):
        status, detail = archive_reference(reference, args)
        if status == "downloaded":
            downloaded += 1
            print(f"downloaded: {reference.path} -> {detail}")
        elif status == "found":
            found += 1
            print(f"found: {reference.path} -> {detail}")
        elif status == "error":
            errors += 1
            print(f"error: {reference.path} -> {detail}", file=sys.stderr)
        elif args.verbose and status != "skipped":
            print(f"{status}: {reference.path} -> {detail}")

        if args.max_downloads and downloaded >= args.max_downloads:
            break

    if args.dry_run:
        print(f"summary: found {found} PDF candidates")
    else:
        print(f"summary: downloaded {downloaded} PDFs")
    if errors and args.strict:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
