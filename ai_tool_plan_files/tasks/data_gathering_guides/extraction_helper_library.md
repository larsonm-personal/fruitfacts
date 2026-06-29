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
  - Preserve either layout text or cleaned raw line text depending on source
    shape
- `pdf_table_tools.py`
  - Bounded PDF table sections, fixed-width row slicing, row continuation, and
    row-start detection from either the name column or a separate row-number
    column
  - Raw `pdftotext` numbered block parsing for PDF tables that extract as
    `1.`, name, zones, regions, and tail lines rather than a horizontal table
  - Flow-mode PDF catalog entry parsing where a heading sets the crop/category
    and each following all-caps cultivar name starts a description line
  - Wrapped PDF bullet-list parsing where headings set crop/category and
    indented continuation lines finish parenthesized cultivar notes
  - PDF quoted-entry parsing where cultivar paragraphs start with quoted names,
    including same-line entry splitting and repeated-name merging
  - Tail parsers for first-token splits such as `Fresh Dessert`, first-line
    prefix plus description, and self-fruitful token splits
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
  - Heading-aware HTML list items and unheaded HTML name-matrix tables for
    extension pages where recommendations are nested under crop headings or
    displayed as compact cultivar grids
  - Uses strict JSON config files under `helper_scripts/extraction_configs/`
    so the Python standard library can parse them
- `extract_source.py`
  - A manifest runner for named config-backed extractors listed in
    `helper_scripts/extraction_manifest.json`
  - Keeps one-source stdout output while avoiding one tiny Python wrapper per
    config

Config-backed sources should be listed in `extraction_manifest.json` and run
through `extract_source.py`. Keep new source-specific parsing in Python only
when the source shape cannot be expressed clearly in config.

For regular sources, prefer a config file before adding another source-specific
script. The config runner currently supports:

- HTML sources fetched with the shared browser-like user agent
- Born-digital PDF sources extracted through `pdftotext`
- Simple tables where the first row is the header
- Tables with source title rows before the real header row
- Required-header table selection
- Explicit table-index selection with required-header validation for sources
  where one table's headers are a subset of another table's headers
- Key-column filtering for footnotes and note rows
- Table transforms for leading rowspans, leading name fragments, and section
  rows
- PDF marker lists inside bounded sections
- PDF fixed-width tables inside bounded sections
- PDF numbered blocks from raw `pdftotext` output, with configurable zone,
  region, tail, and skip patterns
- PDF catalog entries from flow-mode `pdftotext` output, with configurable
  category heading rules, smart titlecase name repair, and continuation lines
- PDF bullet lists with configurable heading rules, continuation indentation,
  skip rules, row overrides, and partial parenthetical entry repair
- PDF quoted cultivar entries with configurable quote characters, skipped
  figure/page lines, source-local initial-letter spacing repair, and repeated
  quoted-name merging
- HTML paragraph blocks where each useful paragraph starts with a quoted
  cultivar name
- HTML paragraph blocks where each useful paragraph starts with `Name:`
- HTML list items where `Name: description` entries inherit crop/category
  context from `h2`, `h3`, and `h4` headings
- HTML name-matrix tables where each cell is a cultivar name, including
  optional group labels, suffix-to-type mapping, and generated source notes
- Lookup tables keyed by cultivar, such as disease rating tables
- Declarative row splits for source rows that clearly contain two varieties
- Generated rows for source notes that explicitly name a small fixed set of
  varieties
- Optional category, location, harvest, and labelled description mapping
- Prefix-prioritized harvest sentence selection for catalog sources where
  useful timing sentences start with phrases such as `Fruit ripens` or
  `Harvest starting`
- Description parts that turn source flag columns such as `X` under use columns
  into readable labelled text
- Name overrides, `AKA` values, and trailing footnote-marker stripping
- Row overrides for source rows where PDF extraction splits a name or moves a
  word into the wrong field
- Top-level config `text_fixes` shared by every extractor in one source

The worked UGA C740 and C742 configs show the ideal direction: no
source-specific Python file, only a source config that drives the shared
extractor. Named manifest entries are preferred over compatibility wrappers.

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
- UGA B807 bunch grapes: two clean HTML tables where the second table's
  headers are a subset of the first, handled with explicit table indexes and
  flag-column description parts
- UGA C766 caneberries: grouped HTML table where fruit type is carried down
  through short rows before mapping rows to blackberry or raspberry records
- USU apple recommendations: clean HTML table with local name harmonization,
  `AKA` fields, and source suffix notes for table footnote markers
- Purdue HO-44-W and HO-46-W small-fruit pages: compact HTML cultivar
  recommendation paragraphs represented as fixed generated row groups in
  config, without source-specific Python
- UMD EB-2023-0684 apples: source offered both a PDF and a landing page, and
  the landing page's clean HTML table was preferred over the PDF layout table
  for the generated reference
- UNL G2354 fruit tree cultivars: raw `pdftotext` numbered blocks with
  pollinizer numbers, zones, regions, uses, and descriptions, plus row
  overrides for PDF line-split cultivar names
- UWisc A2582 southern tree and stone fruit: flow-mode PDF catalog entries
  where section headings set categories and cultivar names appear as all-caps
  lead tokens before prose descriptions
- UWyo zone 3 and 4 fruit list: layout-mode PDF bullet entries where category
  headings set crop types, cultivar notes wrap across indented lines, and some
  non-cultivar species bullets need to be skipped
- Texas A&M E-612 stone fruit: raw PDF quoted cultivar entries where
  `pdftotext` splits initial letters from names and some cultivar mentions
  inside descriptions should not become separate records
- MSState P966 fruit and nut recommendations: HTML `li` entries where source
  headings carry the crop, region, season, and astringency context, plus
  separate HTML peach tables and a generated-row pecan home-planting list
- NDSU FN590 jams and jellies: unheaded HTML cultivar grids embedded in a food
  preservation publication, with group cells for raspberry bearing type,
  suffix-to-type mapping for Prunus names, and prose-generated Juneberry rows

## Manifest Configs

Config-backed extractors are registered in
`helper_scripts/extraction_manifest.json`. Run one by ID:

```powershell
python helper_scripts/extract_source.py umaine_2172_caneberries
```

List available IDs:

```powershell
python helper_scripts/extract_source.py --list
```

Run several entries to files:

```powershell
python helper_scripts/extract_source.py --all --output-dir $env:TEMP\fruitfacts_drafts
```

The UMaine 2172 and 2184 conversions added config support for colon-led
narrative paragraphs, category heading cleanup, ordered harvest phrase maps,
`AKA` values from name overrides, generated rows, and table-summary lookups.

## PDF Lessons

- Prefer `pdftotext -layout` first for born-digital PDFs
- For two-column PDFs, test `pdftotext -raw` before writing column repair code
- For dense table PDFs, compare `pdftotext -layout` and raw output. Layout may
  preserve columns but split words; raw may preserve sentences but turn rows
  into numbered blocks
- If raw PDF output transposes a table into all names, then all zones, then all
  descriptions, keep that table out of the first draft unless a separate parser
  or manual review pass is justified
- Parse bounded sections rather than stopping at the first period because names
  such as `A.C. Wendy` contain punctuation
- Treat source spellings and extraction artifacts separately. If a likely source
  typo is normalized, keep the source spelling as `AKA` or in a note and leave
  `needs_help`
- For catalog-like PDFs, try flow-mode text when layout text is column-heavy.
  Category headings can be exact text or prefixes, and prefix rules may parse
  the remainder of a line when a heading and first cultivar land together
- For catalog harvest timing, prefer source-leading sentence prefixes such as
  `Fruit ripens`, `Harvest beginning`, or `Harvest starting` over broad
  substring searches. Broad searches can grab comparison sentences instead of
  the actual timing sentence
- For wrapped bullet-list PDFs, preserve indentation until after continuation
  decisions. Collapsing whitespace too early can glue unrelated source notes
  onto the previous cultivar.
- For quoted-entry PDFs, split only likely entry-start quotes, such as quotes at
  the beginning of a line or after a sentence boundary. Cultivar names quoted
  inside descriptions are usually pollinizers, parents, or examples, not new
  rows.
- Do not commit downloaded PDFs directly unless the DVC asset workflow is being
  used

## HTML Lessons

- When extension pages expose both a PDF and a clean HTML publication, inspect
  the HTML first. P966's HTML list items and peach tables were cleaner than
  parsing the PDF paragraphs.
- For list pages where headings carry the source meaning, keep the heading
  rules in config. This makes category switches such as Mississippi's
  non-astringent and astringent persimmon seasons reviewable.
- For unheaded cultivar-grid tables, treat group labels and suffixes as parser
  inputs rather than cultivar names. NDSU FN590 uses cells such as `Summer
  Bearing`, `Alderman Plum`, and `Bali Sour Cherry`, each of which needs a
  different row interpretation.
- If a source cell contains a regional caveat in parentheses, keep the cultivar
  name clean and move the caveat to a source note.

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
