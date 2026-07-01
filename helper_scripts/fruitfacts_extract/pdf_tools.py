"""PDF text extraction helpers built around pdftotext"""

import shutil
import subprocess
import tempfile
import os
from pathlib import Path
from urllib.request import Request, urlopen

from fruitfacts_extract.text_tools import DEFAULT_USER_AGENT, clean_text


def fetch_url_bytes(url, user_agent=None, timeout=60):
    if user_agent is None:
        user_agent = DEFAULT_USER_AGENT
    request = Request(url, headers={"User-Agent": user_agent})
    try:
        return urlopen(request, timeout=timeout).read()
    except Exception as original_error:
        return fetch_url_bytes_with_powershell(url, timeout, original_error, user_agent)


def fetch_url_bytes_with_powershell(url, timeout, original_error, user_agent):
    powershell = shutil.which("pwsh") or shutil.which("powershell")
    if not powershell:
        raise original_error
    with tempfile.TemporaryDirectory(prefix="fruitfacts_pdf_fetch_") as temp_dir:
        pdf_path = Path(temp_dir) / "source.pdf"
        script_path = Path(temp_dir) / "fetch.ps1"
        script_path.write_text(
            "\n".join(
                [
                    "param($Url, $OutFile, $TimeoutSec, $UserAgent)",
                    "$ProgressPreference='SilentlyContinue'",
                    "Invoke-WebRequest -Uri $Url -UseBasicParsing "
                    + "-MaximumRedirection 5 -TimeoutSec ([int]$TimeoutSec) "
                    + "-UserAgent $UserAgent "
                    + "-OutFile $OutFile",
                ]
            ),
            encoding="utf-8",
        )
        try:
            subprocess.run(
                [
                    powershell,
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    str(script_path),
                    url,
                    str(pdf_path),
                    str(timeout),
                    user_agent,
                ],
                check=True,
                capture_output=True,
                text=True,
            )
        except Exception:
            raise original_error
        return pdf_path.read_bytes()


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


def pdf_url_to_text(url, layout=True, user_agent=None):
    with tempfile.TemporaryDirectory(prefix="fruitfacts_pdf_") as temp_dir:
        pdf_path = Path(temp_dir) / "source.pdf"
        pdf_path.write_bytes(fetch_url_bytes(url, user_agent=user_agent))
        return pdf_file_to_text(pdf_path, layout=layout)


def clean_pdf_text(text):
    lines = [clean_text(line) for line in text.splitlines()]
    return "\n".join(line for line in lines if line)


def clean_pdf_layout_text(text):
    lines = [clean_text(line, collapse_whitespace=False).rstrip() for line in text.splitlines()]
    return "\n".join(line for line in lines if line.strip())
