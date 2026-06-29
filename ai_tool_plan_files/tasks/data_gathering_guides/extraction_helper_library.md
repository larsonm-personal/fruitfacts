# Extraction Helper Library

## Goal

Grow source-specific extraction scripts into a small, composable helper library
without hiding source judgment inside generic code.

The library should make it easy to turn HTML pages and PDFs into reviewable
draft records. The curated JSON5 file still needs human review, source
interpretation, and project-specific field choices.

## Current Shape

The first shared helpers live under `helper_scripts/fruitfacts_extract/`:

- `text_tools.py`
  - ASCII cleanup, whitespace normalization, simple source-list splitting, and
    labelled table-cell description joining
  - Bounded section extraction and marker-based name-list extraction for PDF
    text
  - Sentence splitting, first matching sentence lookup, and quoted-name
    paragraph parsing for narrative cultivar pages
- `html_tools.py`
  - HTML block and table extraction with script/style noise skipped
  - Heading-bounded block slices for pages where useful source content sits
    between named sections
- `table_tools.py`
  - Simple table header normalization, title-row skipping, and table rows as
    dictionaries
  - Key-column filtering for blank or note rows and leading group-cell repair
    for simple rowspan tables
  - Leading fragment-row repair and section-row retention for common HTML
    tables produced from extension publication systems
- `pdf_tools.py`
  - Download to temp storage and run `pdftotext`
- `record_tools.py`
  - Source-name normalization, source-note appending, plant-record creation,
    row-to-plant conversion, labelled row descriptions, and parser-config
    category cleanup
- `json5_draft.py`
  - JSON-safe quoting and reusable draft reference emission, including optional
    top-level locations and categories

Source-specific scripts such as `extract_umaine_2172.py`,
`extract_umaine_2184.py`, and `extract_csu_763.py` should import these helpers
and keep only the source-specific rules locally.

## Boundary

Shared helpers should do:

- Fetch source bytes or source HTML with a browser-like user agent
- Convert HTML into `(tag, text)` blocks and table rows
- Convert born-digital PDFs into cleaned text or lines
- Normalize common web and PDF punctuation to printable ASCII
- Provide small parsing helpers for recurring text shapes
- Emit draft JSON5 safely enough for review
- Build ordinary draft plant records from source-specific row decisions

Source-specific scripts should do:

- Decide where useful source content starts and stops
- Interpret headings as categories
- Decide plant type, region, and metadata
- Resolve likely aliases or source typos with `needs_help`
- Choose which source wording becomes `description`
- Decide whether timing belongs in `harvest_time_unparsed`

## Worked Patterns

- UMaine 2172: HTML headings plus `Name: description` paragraphs
- UMaine 2184: HTML season headings plus narrative paragraphs plus summary
  table rows
- CSU GardenNotes 763: PDF text plus bounded sections containing suggested
  cultivar lists
- CSU GardenNotes 762: same PDF section/list helpers as GardenNotes 763, with
  category config stripped before draft emission
- Penn State non-scab apple table: HTML table rows mapped by normalized headers
  such as variety, characteristics, and ripening period
- UMaine 2068 peach table: HTML table with leading group rowspans repaired
  before header mapping
- UMaine 2253 blueberry table: compact HTML cultivar table mapped by normalized
  headers, with labelled table cells joined into a draft description
- OSU HYG-1401 and HYG-1422 cultivar tables: simple Ohioline HTML tables with
  occasional source-name normalization and trailing note rows skipped by key
  column
- OSU HYG-1423 grape tables: Ohioline HTML tables with a title row before the
  real column header row, plus a second table keyed by cultivar
- VCE 422-023 apple table: HTML cultivar table with several multi-word cultivar
  names split into leading fragment rows before the full data row
- VCE 422-019 peach and nectarine table: HTML cultivar table with single-cell
  section rows and one combined source row that stays as a local repair
- VCE 422-018 cherry narrative: HTML heading-bounded cultivar paragraphs where
  each useful paragraph starts with a quoted cultivar name
- CSU GardenNotes 764 grape lists: PDF marker lists where cultivar names such
  as `St. Theresa` and `St. Croix` contain periods, so scripts need explicit
  end markers rather than stopping at the first period

## PDF Lessons

- Prefer `pdftotext -layout` first for born-digital PDFs
- For two-column PDFs, test `pdftotext -raw` before writing column repair code
- Parse bounded sections rather than stopping at the first period because names
  such as `A.C. Wendy` contain punctuation
- Treat source spellings and extraction artifacts separately. If a likely source
  typo is normalized, keep the source spelling as `AKA` or in a note and leave
  `needs_help`
- Do not commit downloaded PDFs directly unless the DVC asset workflow is being
  used

## Next Library Steps

1. Add a PDF diagnostic command that reports page count, text length, and
   whether `pdftotext -layout` or `pdftotext -raw` looks cleaner.
2. Add parser-backed checks for draft extractors after the library shape
   stabilizes.
3. Consider a small regression test for helper scripts that compares extracted
   plant counts for representative HTML and PDF sources.
4. Consider moving repeated source metadata into a small data object only if the
   current `REFERENCE_FIELDS` pattern starts to drift.
5. Keep parsers source-specific until a pattern appears in multiple unrelated
   sources.
