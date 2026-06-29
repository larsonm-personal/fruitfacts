#!/usr/bin/env python3
"""Run config-backed extractors from the extraction manifest"""

import argparse
import contextlib
import json
import sys
from pathlib import Path

from extract_from_config import emit_config


DEFAULT_MANIFEST = Path(__file__).with_name("extraction_manifest.json")


def load_manifest(path):
    manifest = json.loads(path.read_text(encoding="utf-8"))
    entries = manifest.get("extractors", [])
    seen = set()
    for entry in entries:
        id_ = entry.get("id")
        config = entry.get("config")
        if not id_ or not config:
            raise ValueError("Each manifest entry needs id and config")
        if id_ in seen:
            raise ValueError(f"Duplicate source id: {id_}")
        seen.add(id_)
    return entries


def entry_map(entries):
    return {entry["id"]: entry for entry in entries}


def resolve_config(manifest_path, entry):
    path = Path(entry["config"])
    if not path.is_absolute():
        path = manifest_path.parent / path
    return path


def draft_path(output_dir, source_id):
    return output_dir / f"{source_id}.json5"


def run_entry(manifest_path, entry, output_dir=None):
    if output_dir is None:
        emit_config(resolve_config(manifest_path, entry))
        return
    output_dir.mkdir(parents=True, exist_ok=True)
    with draft_path(output_dir, entry["id"]).open("w", encoding="utf-8") as output:
        with contextlib.redirect_stdout(output):
            emit_config(resolve_config(manifest_path, entry))


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Run FruitFacts extraction configs listed in a manifest"
    )
    parser.add_argument("source_ids", nargs="*", help="Manifest source IDs to run")
    parser.add_argument("--all", action="store_true", help="Run every manifest entry")
    parser.add_argument("--list", action="store_true", help="List manifest source IDs")
    parser.add_argument(
        "--manifest",
        type=Path,
        default=DEFAULT_MANIFEST,
        help="Path to the extraction manifest",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        help="Write drafts to this directory instead of stdout",
    )
    args = parser.parse_args(argv)

    try:
        manifest_path = args.manifest
        entries = load_manifest(manifest_path)
        by_id = entry_map(entries)

        if args.list:
            for entry in entries:
                print(f"{entry['id']}\t{entry['config']}")
            return 0

        selected = entries if args.all else [by_id[id_] for id_ in args.source_ids]
        if not selected:
            print("Use --list or pass a source ID", file=sys.stderr)
            return 2
        if len(selected) > 1 and args.output_dir is None:
            print("Use --output-dir when running more than one source", file=sys.stderr)
            return 2

        for entry in selected:
            run_entry(manifest_path, entry, output_dir=args.output_dir)
    except KeyError as error:
        print(f"Unknown source ID: {error.args[0]}", file=sys.stderr)
        return 1
    except Exception as error:
        print(f"Failed to extract source: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
