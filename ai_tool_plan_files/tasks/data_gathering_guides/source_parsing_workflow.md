# Source Parsing Workflow

## Goal

Turn one candidate source into a FruitFacts JSON5 reference while leaving behind
enough tooling and notes to make the next source easier.

## Choosing A First Source

Prefer sources with one of these shapes:

- HTML headings followed by repeated `Cultivar: description` paragraphs
- HTML tables with real `tr`, `th`, and `td` tags
- Born-digital PDFs where `pdftotext -layout` keeps rows aligned
- PDF tables with plain text, not scanned page images

Avoid image-only scans and complex multi-column PDFs for the first pass unless
the goal is OCR work.

The first worked example used UMaine Bulletin 2172 because the page has stable
HTML headings and repeated cultivar paragraphs. It did not use table tags, but
the text blocks were regular enough to parse.

## Tools Used

- `rg` for finding similar existing reference files and checking for duplicate
  source titles
- `Invoke-WebRequest` for quick URL status checks and HTML inspection
- Python `html.parser` from the standard library for dependency-free block
  extraction
- Python `html.parser` table hooks for simple HTML comparison tables
- `fruitfacts_extract.table_tools` for mapping simple table rows to dictionaries
  by normalized header names
- A browser-like `User-Agent` header for sources that reject default Python
  urllib requests
- `pdftotext -layout` for PDF title-page and table sanity checks
- PowerShell plus `ConvertFrom-Json` for lightweight JSON5-shape checks after
  converting unquoted keys in memory
- Backend import tests when the change risk justifies the slower validation

## Helper Script Pattern

Source-specific helpers should normally:

1. Fetch or read the source.
2. Extract repeatable blocks such as headings, table rows, or cultivar
   paragraphs.
3. Print a draft to stdout rather than writing repo files directly.
4. Keep the extraction mechanical.
5. Leave judgment, paraphrase, field selection, and source interpretation for
   the curated JSON5 edit.

Example:

```powershell
python helper_scripts/extract_umaine_2172.py > $env:TEMP\umaine_2172_draft.json5
```

The helper output is a draft. Do not treat it as reviewed data.

Shared code belongs in `helper_scripts/fruitfacts_extract/` when at least two
source scripts need the same behavior. Keep source interpretation in the
source-specific script. See
[Extraction Helper Library](extraction_helper_library.md) for the current
library boundary.

After the first several worked examples, the repeated pieces were moved into
shared helpers:

- `json5_draft.emit_reference()` for draft output
- `record_tools.plant_record()` and `record_tools.normalized_name()` for
  common plant-row mechanics
- `text_tools.section_between()` and `text_tools.names_after_marker()` for
  born-digital PDF section parsing
- `table_tools.fill_leading_group_cells()` for simple HTML tables where a
  leading source group was represented by rowspans

## The Art

- Capture source metadata first: title, author, URL, publication dates,
  accessed date, source type, and location.
- Preserve source order unless another local file gives a stronger pattern.
- Use top-level `categories` when source headings carry real meaning.
- Use `category` on each plant to retain table section context.
- Put vague timing in `harvest_time_unparsed`, or in `description`, rather than
  inventing exact dates.
- Prefer concise paraphrases over copied source paragraphs.
- Keep disease, hardiness, flavor, fruit quality, and commercial suitability in
  `description` unless the source gives a clean structure that matches existing
  fields.
- Add `needs_help: true` when photos, ambiguous names, source omissions, or
  possible aliases still need review.
- Do not clear the candidate queue entry silently. Mark it `encoded` and note
  the reference path and helper script.

## UMaine 2172 Notes

The UMaine page was parsed from HTML, not PDF. The helper script:

- Ignores script, style, and footer noise
- Extracts headings and paragraphs
- Detects source categories such as red summer-bearing raspberries
- Pulls `Name: description` cultivar paragraphs into draft plant records
- Converts common web punctuation to ASCII
- Emits JSON5-like output to stdout for review

The committed reference file was then curated by hand. Long source paragraphs
were compressed into concise descriptions, and vague ripening phrases were kept
as `harvest_time_unparsed`.

## UMaine 2184 Notes

The UMaine strawberry page uses a related but slightly different structure:

- Season headings such as Early Season, Midseason, and Day-Neutral
- Cultivar narrative paragraphs, sometimes split as a name-only paragraph
  followed by a description paragraph
- A summary table with variety, ripening time, pest resistance, and comments

The helper script parses both the narrative blocks and the summary table, then
merges table notes into the draft plant records. The curated reference keeps the
season heading as each plant `category`, uses table ripening values as
`harvest_time_unparsed` where appropriate, and keeps disease resistance and
home-garden or plasticulture notes in concise descriptions.

## UMaine 2253 Notes

The UMaine highbush blueberry page is a compact HTML table source. The cultivar
table has headers `Variety`, `Plant Characteristics`, `Fruit Qualities`, and
`Ripening Season`.

The helper script uses `table_tools.find_table()` and `table_tools.table_to_dicts()`
for row extraction, then uses `text_tools.join_labelled_values()` to build a
mechanical draft description from descriptive columns. The curated reference
compresses those labelled draft sentences into more natural source notes while
keeping the table ripening values in `harvest_time_unparsed`.

This source also showed a small name-harmonization case. UMaine prints
`Blue Gold`, while existing FruitFacts data uses `Bluegold`. The curated file
keeps the existing canonical name and notes the source spelling in the
description.

## UMaine 2068 Notes

The UMaine peach page is an HTML table source, but the first `Type` column uses
rowspans. The simple HTML table parser sees the first row in a group as full
width and later rows as one cell short. The helper script repairs this with
`table_tools.fill_leading_group_cells()` before calling
`table_tools.table_to_dicts()`.

The source uses asterisks to mark varieties evaluated at the University of
Maine Highmoor Farm. The extractor removes the asterisk from the plant name and
keeps the evaluation marker in the draft description.

## CSU GardenNotes 763 Notes

This was the first PDF worked example. The source is short and born-digital, so
`pdftotext -layout` produced useful text without OCR. The source shape is not a
table; it has bounded sections where sentences say `Suggested cultivars
include ...`.

The helper script uses shared PDF text extraction and then source-specific
section parsing. A first naive regex stopped at the period inside `A.C. Wendy`,
so the parser now bounds each category by the next section heading before
splitting cultivar names.

## CSU GardenNotes 762 Notes

The blackberry PDF uses the same broad shape as GardenNotes 763: bounded text
sections with sentences such as `Suggested cultivars include ...`. The helper
script uses `text_tools.section_between()` and `text_tools.names_after_marker()`
for those repeated pieces.

Unlike the strawberry PDF, some category config fields such as `start`, `end`,
and `marker` are parser instructions, not source data. The script passes the
category config through `record_tools.category_records()` before emitting the
draft so only `name` and `description` are printed.

## Penn State Non-Scab Apple Table Notes

The Penn State page is a compact HTML table with headers `Variety`,
`Characteristics`, and `Ripening Period`. The helper script uses
`table_tools.find_table()` and `table_tools.table_to_dicts()` so the
source-specific parser can refer to `row["variety"]`,
`row["characteristics"]`, and `row["ripening_period"]`.

This source also showed why local name harmonization belongs in
source-specific code. The table uses common or trademark-facing names such as
Zestar!, Ginger Gold, Blondee, Cameo, and SunCrisp; the curated file maps those
to existing FruitFacts canonical names and keeps the source names in
descriptions.

## OSU HYG-1401 Apple Notes

The Ohioline apple page has one compact cultivar table with headers
`Cultivar`, `Bloom Season`, `Ripening Season`, and `Description`. The helper
script uses `table_tools.find_table()`, `table_tools.table_to_dicts()`, and
`table_tools.keyed_data_rows()`.

This source has several names that need local FruitFacts normalization:
`Pristine` becomes `Co-op 32`, `Pixie Crunch` becomes `Co-op 33`,
`William's Pride` becomes `Williams' Pride`, and `Goldrush` becomes
`GoldRush`. The source spelling is kept in each curated description. This is a
good example of why name normalization belongs in source-specific scripts until
there is a reviewed alias authority.

## OSU HYG-1422 Blueberry Notes

The Ohioline blueberry page has one cultivar table with headers `Cultivar`,
`Ripening Season`, `Yield`, `Fruit Size`, `Fruit Quality`, and `Remarks`.
It ends with a `Note:` row, which should not become a plant record. The helper
script uses `table_tools.keyed_data_rows()` to drop that row after converting
the table to dictionaries.

The source gives relative ratings rather than measured yield or fruit-size
values. The curated reference keeps those ratings in descriptions and preserves
the vague season values as `harvest_time_unparsed`.

## OSU HYG-1423 Grape Notes

The Ohioline grape page has several HTML tables where the first row is a table
title and the second row is the real header. The helper script uses
`table_tools.find_table_with_header_row()` so it can find the actual header row
inside each table.

The source also has a separate disease-susceptibility table keyed by cultivar.
The parser builds a disease lookup and appends the source's star ratings to
each plant description. The curated file keeps those ratings in text for now
rather than inventing a grape-specific disease schema.
