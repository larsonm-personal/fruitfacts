"""PDF text extraction helpers built around pdftotext"""

import shutil
import subprocess
import tempfile
import os
from pathlib import Path
from urllib.request import Request, urlopen

from fruitfacts_extract.text_tools import DEFAULT_USER_AGENT, clean_text


def fetch_url_bytes(url, user_agent=DEFAULT_USER_AGENT, timeout=60):
    request = Request(url, headers={"User-Agent": user_agent})
    return urlopen(request, timeout=timeout).read()


def pdftotext_path():
    path = shutil.which("pdftotext")
    if not path:
        raise RuntimeError("pdftotext was not found on PATH")
    return path


def pdf_file_to_text(pdf_path, layout=True):
    file_handle, text_name = tempfile.mkstemp(suffix=".txt")
    os.close(file_handle)
    text_path = Path(text_name)
    try:
        command = [pdftotext_path()]
        if layout:
            command.append("-layout")
        command.extend([str(pdf_path), str(text_path)])
        subprocess.run(command, check=True, capture_output=True, text=True)
        return text_path.read_text(encoding="utf-8", errors="replace")
    finally:
        text_path.unlink(missing_ok=True)


def pdf_url_to_text(url, layout=True):
    with tempfile.TemporaryDirectory(prefix="fruitfacts_pdf_") as temp_dir:
        pdf_path = Path(temp_dir) / "source.pdf"
        pdf_path.write_bytes(fetch_url_bytes(url))
        return pdf_file_to_text(pdf_path, layout=layout)


def clean_pdf_text(text):
    lines = [clean_text(line) for line in text.splitlines()]
    return "\n".join(line for line in lines if line)


def clean_pdf_layout_text(text):
    lines = [clean_text(line, collapse_whitespace=False).rstrip() for line in text.splitlines()]
    return "\n".join(line for line in lines if line.strip())
