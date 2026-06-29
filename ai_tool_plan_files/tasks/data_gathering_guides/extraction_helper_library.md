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
- `extract_from_config.py`
  - A config runner for simple sources that can be expressed as source
    metadata plus table, paragraph, or marker-list mappings
  - Uses strict JSON config files under `helper_scripts/extraction_configs/`
    so the Python standard library can parse them

Most source-specific scripts should now be tiny compatibility wrappers around
`extract_from_config.py`. Keep new source-specific parsing in Python only when
the source shape cannot be expressed clearly in config.

For regular sources, prefer a config file before adding another source-specific
script. The config runner currently supports:

- HTML sources fetched with the shared browser-like user agent
- Born-digital PDF sources extracted through `pdftotext`
- Simple tables where the first row is the header
- Tables with source title rows before the real header row
- Required-header table selection
- Key-column filtering for footnotes and note rows
- Table transforms for leading rowspans, leading name fragments, and section
  rows
- PDF marker lists inside bounded sections
- HTML paragraph blocks where each useful paragraph starts with a quoted
  cultivar name
- HTML paragraph blocks where each useful paragraph starts with `Name:`
- Lookup tables keyed by cultivar, such as disease rating tables
- Declarative row splits for source rows that clearly contain two varieties
- Generated rows for source notes that explicitly name a small fixed set of
  varieties
- Optional category, location, harvest, and labelled description mapping
- Name overrides, `AKA` values, and trailing footnote-marker stripping

The worked UGA C740 and C742 configs show the ideal direction: no
source-specific Python file, only a source config that drives the shared
extractor. Older command names are kept as wrappers where useful for continuity.

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
- Stay in source-specific Python when the source needs nontrivial narrative
  merging or source-specific judgment that has not repeated elsewhere

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
- UGA C740 apples and C742 pears: config-driven HTML table extraction with
  title-row tables, footnote-row skipping, source name overrides, and trailing
  footnote-marker stripping

## Converted Configs

These earlier source-specific scripts now have strict JSON configs that
reproduce the old script stdout exactly:

- `extract_csu_762.py` -> `extraction_configs/csu_762_blackberries.json`
- `extract_csu_763.py` -> `extraction_configs/csu_763_strawberries.json`
- `extract_csu_764.py` -> `extraction_configs/csu_764_grapes.json`
- `extract_osu_hyg_1401_apples.py` -> `extraction_configs/osu_hyg_1401_apples.json`
- `extract_osu_hyg_1422_blueberries.py` -> `extraction_configs/osu_hyg_1422_blueberries.json`
- `extract_osu_hyg_1423_grapes.py` -> `extraction_configs/osu_hyg_1423_grapes.json`
- `extract_psu_non_scab_apples.py` -> `extraction_configs/psu_non_scab_apples.json`
- `extract_umaine_2068.py` -> `extraction_configs/umaine_2068_peaches.json`
- `extract_umaine_2172.py` -> `extraction_configs/umaine_2172_caneberries.json`
- `extract_umaine_2184.py` -> `extraction_configs/umaine_2184_strawberries.json`
- `extract_umaine_2253.py` -> `extraction_configs/umaine_2253_blueberries.json`
- `extract_vce_422_018_cherries.py` -> `extraction_configs/vce_422_018_cherries.json`
- `extract_vce_422_019_peaches.py` -> `extraction_configs/vce_422_019_peaches.json`
- `extract_vce_422_023_apples.py` -> `extraction_configs/vce_422_023_apples.json`

The UMaine 2172 and 2184 conversions added config support for colon-led
narrative paragraphs, category heading cleanup, ordered harvest phrase maps,
`AKA` values from name overrides, generated rows, and table-summary lookups.

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
5. Add a small compatibility check script that compares each config-backed
   wrapper against a saved golden draft output.
6. Keep parsers source-specific when a pattern has not appeared in multiple
   unrelated sources or when the extraction requires source judgment.
