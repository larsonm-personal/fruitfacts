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

When the source is a regular HTML table, a bounded PDF marker list, a simple
quoted-paragraph narrative, or a `Name: description` paragraph list, first try
a config-driven extraction instead of adding a new per-source Python file:

```powershell
python helper_scripts/extract_from_config.py helper_scripts/extraction_configs/uga_c740_apples.json > $env:TEMP\uga_c740_apples.json5
```

This is the preferred shape for an eventual omniparser. The config should name
the source, reference fields, locations, categories, name overrides, extraction
shape, key columns, harvest source fields, and description fields. The shared
runner should own recurring mechanical repairs such as skipping note rows,
stripping trailing footnote markers from plant names, merging wrapped table
name rows, splitting a clearly combined source row, or turning a source note
into a fixed set of generated rows. Curated JSON5 review still owns source
judgment.

After the first several worked examples, the repeated pieces were moved into
shared helpers:

- `json5_draft.emit_reference()` for draft output
- `record_tools.plant_record()` and `record_tools.normalized_name()` for
  common plant-row mechanics
- `text_tools.section_between()` and `text_tools.names_after_marker()` for
  born-digital PDF section parsing
- `table_tools.fill_leading_group_cells()` for simple HTML tables where a
  leading source group was represented by rowspans
- `table_tools.merge_leading_fragment_rows()` for tables where a wrapped name
  appears as a name-only row followed by the rest of the row
- `table_tools.table_to_dicts_with_sections()` for tables with single-cell
  section rows between groups of ordinary data rows
- `record_tools.plant_records_from_rows()` for the common case where parsed
  rows map directly to draft plant records
- `extract_from_config.py` for sources that can be represented as source
  metadata plus table, paragraph, or marker-list mappings

Existing command names can stay as tiny wrappers around a config. This keeps
old notes and shell history working while making the real source-specific logic
declarative.

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

The UMaine page is parsed from HTML, not PDF. The config-backed helper:

- Ignores script, style, and footer noise
- Extracts headings and `Name: description` paragraphs
- Detects source categories such as red summer-bearing raspberries
- Pulls cultivar paragraphs into draft plant records
- Converts common web punctuation to ASCII
- Emits JSON5-like output to stdout for review

The committed reference file was then curated by hand. Long source paragraphs
were compressed into concise descriptions, and vague ripening phrases were kept
as `harvest_time_unparsed`.

The config runner now handles this source's heading cleanup, static category
descriptions, the special everbearing blackberry note that generates four
fixed rows, the `Fall Gold` to `Fallgold` name override with `AKA`, and the
hand-written harvest phrase map.

## UMaine 2184 Notes

The UMaine strawberry page uses a related but slightly different structure:

- Season headings such as Early Season, Midseason, and Day-Neutral
- Cultivar narrative paragraphs, sometimes split as a name-only paragraph
  followed by a description paragraph
- A summary table with variety, ripening time, pest resistance, and comments

The config runner parses both the narrative blocks and the summary table, then
merges table notes into the draft plant records. The curated reference keeps
the season heading as each plant `category`, uses table ripening values as
`harvest_time_unparsed` where appropriate, and keeps disease resistance and
home-garden or plasticulture notes in concise descriptions.

This source added support for `Name:` paragraphs whose descriptions sometimes
come from the following paragraph, plus lookup-derived description fragments
from a summary table.

## UMaine 2253 Notes

The UMaine highbush blueberry page is a compact HTML table source. The cultivar
table has headers `Variety`, `Plant Characteristics`, `Fruit Qualities`, and
`Ripening Season`.

The config uses `table_tools.find_table()` and `table_tools.table_to_dicts()`
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
width and later rows as one cell short. The config runner repairs this with
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

The config runner now handles this pattern with `pdf_marker_list` extractors.
A first naive regex stopped at the period inside `A.C. Wendy`, so the parser
bounds each category by the next section heading before splitting cultivar
names.

## CSU GardenNotes 762 Notes

The blackberry PDF uses the same broad shape as GardenNotes 763: bounded text
sections with sentences such as `Suggested cultivars include ...`. The helper
config uses `text_tools.section_between()` and `text_tools.names_after_marker()`
through `extract_from_config.py` for those repeated pieces.

Unlike the strawberry PDF, some category config fields such as `start`, `end`,
and `marker` are parser instructions, not source data. The script passes the
category config through `record_tools.category_records()` before emitting the
draft so only `name` and `description` are printed.

## CSU GardenNotes 764 Notes

The grape PDF is another born-digital GardenNotes source, but its cultivar
lists appear in ordinary prose near the start of the document. The helper
config uses `text_tools.section_between()` to isolate `Types of Grapes`, then
uses `text_tools.names_after_marker()` with explicit end markers.

Explicit end markers matter here because source names such as `St. Theresa`
and `St. Croix` contain periods. A parser that simply stops at the next period
would truncate those names.

## Penn State Non-Scab Apple Table Notes

The Penn State page is a compact HTML table with headers `Variety`,
`Characteristics`, and `Ripening Period`. The config uses
`table_tools.find_table()` and `table_tools.table_to_dicts()` so the
runner can refer to `row["variety"]`,
`row["characteristics"]`, and `row["ripening_period"]`.

This source also showed why local name harmonization belongs in source-local
config. The table uses common or trademark-facing names such as
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
title and the second row is the real header. The config runner uses
`table_tools.find_table_with_header_row()` so it can find the actual header row
inside each table.

The source also has a separate disease-susceptibility table keyed by cultivar.
The parser builds a disease lookup and appends the source's star ratings to
each plant description. The curated file keeps those ratings in text for now
rather than inventing a grape-specific disease schema.

## VCE 422-023 Apple Notes

The Virginia apple page has a useful HTML cultivar table, but several names are
split into name-only rows followed by rows with the harvest and trait values.
The config runner repairs that source shape with
`table_tools.merge_leading_fragment_rows()` before converting rows to
dictionaries.

The source also gives table ratings for fresh use and cooking use. The curated
file keeps those source ratings in `description` and explains the rating legend
in the category description. The harvest dates are kept as source dates for
Blacksburg, with the source's Blue Ridge timing caveat left in the category
description.

## VCE 422-019 Peach And Nectarine Notes

The Virginia peach and nectarine page has one HTML cultivar table with
single-cell category rows for white-fleshed peaches and nectarines. The helper
config uses `table_tools.table_to_dicts_with_sections()` to retain that section
context and uses it to choose both plant `category` and `type`.

The official HTML combines Morton and Raritan Rose into one row even though the
PDF text confirms two dates and two descriptions. That repair is now a
declarative row split in the VCE config. The resulting plant descriptions keep
a visible note that the HTML row was combined.

## VCE 422-018 Cherry Notes

The Virginia cherry page is a narrative source rather than a table. Useful
cultivar paragraphs are bounded by the `Tart Cherries` and `Cherry Pollination`
headings, so the config runner uses `html_tools.blocks_between_headings()`.

Each cultivar paragraph starts with a quoted cultivar name. The extractor uses
`text_tools.quoted_name_paragraph()` to split the name from the paragraph, then
uses `text_tools.first_sentence_containing()` to preserve the source's ripening
sentence as `harvest_time_unparsed` when one exists. The source's plain
paragraph label `Dark Sweet Cherries` is treated as a category change even
though it is not marked up as a heading.

## UGA C740 And C742 Config Notes

The UGA apple and pear pages were encoded without source-specific Python
scripts. Each source uses a strict JSON config in
`helper_scripts/extraction_configs/` and the shared
`helper_scripts/extract_from_config.py` runner.

The apple source has one ordinary table and one disease-resistant table. The
first table ends with a footnote row whose key cell starts with `1 Listed`, so
the config skips that prefix. The disease-resistant table uses `Goldrush`; the
config maps it to the existing `GoldRush` capitalization and leaves a source
note in the draft description.

The pear source has source title rows before the real header rows. The config
uses `header_row` plus `title_contains` to find the European and Asian pear
tables. One European pear row has a trailing footnote marker on the cultivar
name, so the shared runner strips trailing note numbers when the config asks
for it.
