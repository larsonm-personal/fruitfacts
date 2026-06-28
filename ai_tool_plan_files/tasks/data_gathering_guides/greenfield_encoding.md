# Greenfield Encoding

## Goal

Create a full JSON5 encoding for a source that does not yet have a reference
file.

## Inputs

- Source file or page
- Existing JSON5 examples for similar source type, crop, or region
- Association pattern helper

## Output

A new JSON5 reference file with metadata, locations, categories, and plant
entries.

## Steps

1. Choose the closest existing JSON5 file as a model.
2. Capture metadata first.
3. Define locations from explicit source context.
4. Encode categories only when they carry meaning beyond the plant type.
5. Encode plant entries in source order unless there is a clear local pattern.
6. Leave `needs_help` when any important section is not yet encoded.

## Tooling Pattern

For parseable HTML or PDF sources, see
[Source Parsing Workflow](source_parsing_workflow.md). Prefer helper scripts
that print drafts to stdout, then curate the committed JSON5 by hand.

## Checks

- The JSON5 parses.
- Locations match the source, not later assumptions.
- Plant names and types match existing conventions where possible.
- Harvest and recommendation wording is not over-normalized.

## Open Questions

- Should every source table become one collection, or can some sources split
  into multiple files?
- How should image-only tables be marked before OCR work is reliable?
