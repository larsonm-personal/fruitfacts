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
python helper_scripts/extract_source.py umaine_2172_caneberries > $env:TEMP\umaine_2172_draft.json5
```

The helper output is a draft. Do not treat it as reviewed data.

Shared code belongs in `helper_scripts/fruitfacts_extract/` when at least two
source scripts need the same behavior. Keep source interpretation in the
source-specific script. See
[Extraction Helper Library](extraction_helper_library.md) for the current
library boundary.

When the source is a regular HTML table, a bounded PDF marker list, a simple
quoted-paragraph narrative, a `Name: description` paragraph list, or cultivar
headings under category headings, first try a config-driven extraction instead
of adding a new per-source Python file:

```powershell
python helper_scripts/extract_source.py uga_c740_apples > $env:TEMP\uga_c740_apples.json5
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
- `extract_source.py` plus `extraction_manifest.json` for named source IDs
  that point at config-backed extractors

Named manifest entries replace tiny source-specific wrappers. This keeps the
real source-specific logic declarative while avoiding one Python file per
config.

The Ontario bramble guide added `html_heading_records` for pages where a
category heading such as `Red raspberry cultivars` is followed by cultivar
headings and paragraph descriptions. Use heading rules to preserve category
context, and use row splits when a source heading clearly combines two
cultivars.

The Ontario blueberry guide reused `html_heading_records` with the record tag
set to `h3`. When non-cultivar section headings share that same tag, use
`skip_names` and add generated rows or a second extractor for cultivar names
embedded together in prose.

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
through `extract_source.py` and `extract_from_config.py` for those repeated
pieces.

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
`helper_scripts/extract_source.py` manifest runner and
`helper_scripts/extract_from_config.py` config runner.

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

## UGA B807 And C766 Config Notes

The UGA bunch grape and caneberry pages continue the same pattern: the useful
data lives in ordinary HTML tables on the `extension.uga.edu` detail pages,
even when the candidate queue found newer Field Report landing pages and PDFs.
The configs therefore use the detail pages as the parse source and keep the
current source URLs in the committed reference metadata.

B807 has two cultivar tables. The second table's headers are a subset of the
first table's headers, so required-header matching alone would select the
first table twice. The config runner now supports `table_index` with
required-header validation for this source shape. The grape use columns are
source flags, so the config uses a `flag_labels` description part to turn
checked columns into text such as `Uses: white wine; fresh eating`.

C766 has one grouped table where the `Fruit Type` cell spans several rows in
the browser. The simple parser sees those following rows as short rows, so the
config uses `fill_leading_group_cells` before mapping `Blackberries` to
`Blackberry` records and `Raspberries` to `Raspberry` records.

## USU Apples And Purdue Small Fruit Notes

The USU apple page is a straightforward HTML table source. The config extracts
the variety table directly, keeps source ripening values in
`harvest_time_unparsed`, and uses `name_suffix_notes` for the source asterisk
that marks Utah commercial cultivars. A few well-established FruitFacts names
are harmonized in config, with source names retained as `AKA` values or source
notes.

The Purdue raspberry and strawberry pages are not table sources. Their useful
cultivar content is in compact recommendation paragraphs under the `Cultivars`
heading. Rather than write custom Python, the configs use
`colon_paragraph_blocks` with an intentionally impossible paragraph regex and
`generated_rows` keyed by source paragraph prefixes. This keeps the source
judgment visible in JSON while still letting the shared runner emit reference
records and compare generated output to committed JSON5.

## UWisc A2582 Southern Fruit Notes

The Wisconsin A2582 PDF is more difficult than the earlier simple lists and
HTML tables. Layout-mode `pdftotext` preserves some columns but splits many
words. Flow-mode text produces a catalog-like stream where a heading sets the
fruit group and most entries start with an all-caps cultivar name followed by
a prose description.

The config uses `pdf_catalog_entries` for the tree-fruit and stone-fruit
sections. The parser keeps a current category from exact or prefix heading
rules, then starts a new row when a line begins with all-caps name tokens and
continues the description across following lines. Source-local `text_fixes`
clean PDF word breaks, while `name_overrides` handles project canonical names
such as `Autumncrisp`, `Co-op 39`, `GoldRush`, and `Sweet Cherry Pie`.

This source also showed why harvest extraction should sometimes be prefix
based. A broad search for `ripens` can catch comparison sentences such as
`the fruit ripens earlier`; the A2582 config instead prefers sentence starts
such as `Fruit ripens`, `Harvest beginning`, and `Harvest starting`.

The committed reference currently encodes the tree-fruit and stone-fruit
catalog sections only. The small-fruit sections remain in the candidate queue
because the PDF flow text interleaves columns more aggressively there and
needs a separate parser strategy.

## UWyo Zone 3 And 4 Fruit List Notes

The Wyoming Extension zone 3 and 4 fruit list is a layout-mode PDF with
source headings such as Apples, Pears, American Plums, and Serviceberries or
Juneberries followed by hyphen bullets. Most cultivar notes are parenthesized,
but several entries wrap before the closing parenthesis. One pear line also
contains two bullet starts on the same extracted line.

The config uses `pdf_bullet_list` with heading rules that set category and
plant type. The helper keeps raw indentation long enough to distinguish
wrapped cultivar-note lines from ordinary source notes such as the Wyoming
Apple Project paragraph. The parser also handles partial parenthetical bullets,
same-line bullets, and source-local row overrides such as treating Kristen as
a sweet cherry.

The committed reference keeps `needs_help` because a few source rows are
species-level rather than cultivar-level, and Red Lake has a visibly truncated
susceptibility note in the official PDF text.

## Texas A&M E-612 Stone Fruit Notes

Texas A&M E-612 is a raw PDF prose source. Cultivar paragraphs start with
quoted names such as `` `M ethley' `` or `` `R oyal Lee' `` after `pdftotext`
splits the first letter from the rest of some names. Some lines contain two
true cultivar entries, while other quoted names inside the sentence are
pollinizers or parents and should remain in the description.

The config uses `pdf_quoted_entries` with source-local repair for spaced
initial letters, figure and page-number skipping, and repeated-name merging.
The parser treats quoted names at the start of a line or after a sentence
boundary as entry starts, while quoted names inside descriptions stay in the
description. The committed reference covers plums, nectarines, apricots, sweet
cherries, and the single almond variety named by the source.

## MSState P966 Fruit And Nut Notes

Mississippi State P966 is a broad HTML publication with a PDF fallback. The
HTML page is easier to parse because most cultivar descriptions are `li` items
in the form `Name: description`, while headings carry crop, region, season, or
persimmon astringency context. The peach recommendations are separate HTML
tables with chilling hours and average maturity dates.

The config uses `html_list_items` for the heading-aware lists, two
`html_table` extractors for the north and south Mississippi peach tables, and
a generated-row paragraph for the home-planting pecan list. It keeps
`needs_help` because the source also has narrative regional pecan lists and
some rows, such as Cardinal strawberry, appear in more than one source group.
Peaches repeated in both north and south tables are merged into one record
with combined regional maturity and chilling-hour notes because the import
schema allows only one collection item per type and name.

This source added optional row dedupe for repeated list entries and showed why
harvest extraction sometimes needs sentence-prefix matching. Broad substring
matching caught phrases such as `ripen properly`, while prefix matching keeps
only sentences that start with timing terms such as `Ripens`.

## Clemson HGIC Blueberry And Bunch Grape Notes

Clemson HGIC 1401 is an HTML list source, but the useful category changes are
ordinary paragraphs rather than section headings. The config uses
`html_list_items` with `context_rules` to switch from rabbiteye to Southern
highbush and to set early, midseason, late, and dwarf container categories
before the following `li` rows. The parser supports `when` checks so repeated
paragraph text such as `Early season cultivars:` can mean different categories
depending on the current blueberry group.

Clemson HGIC 1402 puts all bunch grape cultivar names inside two narrative
paragraphs, with region and use clauses dividing the lists. The config uses
`inline_quoted_names` with repeated `names_after` and `names_before` slices to
preserve those source groups. The extractor can strip terminal punctuation from
quoted names because the source includes commas inside some quotes, such as
`'Mars,'`, and row overrides keep source spellings like `Blanc Du Bois` visible
while generating canonical names.

Clemson HGIC 1358 is a prose marker-list source. The useful cultivar list is a
single sentence under the `Varieties` heading, while nearby prose discusses
species groups, rootstocks, and pollination. The config uses `html_marker_list`
bounded by `Varieties` and `Harvest` to extract only the comma-separated named
cultivars after `Among the more popular varieties are`. These named cultivars
are encoded as `Japanese Plum`; `Species Plum` is not used because the list is
not a natural species listing.

Clemson HGIC 1400 is table-shaped but still benefits from category preservation.
The recommendation table has variety, cane type, fruiting habit, and thorniness
columns. The config maps cane type plus fruiting habit into categories such as
`Erect Floricane blackberries for South Carolina`, while keeping cane type,
fruiting habit, and thorniness as labelled description fields. The source names
`Prime-Ark Freedom` and `Prime-Ark Traveler`; the config harmonizes them to
existing FruitFacts blackberry names with source notes because the importer
uses normalized name strings for uniqueness.

Clemson HGIC 1350 is a straightforward HTML table after the surrounding prose
is ignored. The config uses the numbered headers directly, skips the source
footnote row that begins with `1Listed`, keeps regional area codes and
pollination codes in labelled descriptions, and pulls harvest phrases from the
characteristics text without converting them into exact dates.

Clemson HGIC 1354 is a mixed peach and nectarine table. The source marks
nectarine rows by adding `*` to the variety name and explaining `*Nectarine` in
the footnote row. The shared table parser now applies `name_suffix_type_map` to
ordinary `html_table` rows, so those suffixes are stripped from names and the
affected rows become `Nectarine` records with source notes. The config also
harmonizes source names such as `Junegold`, `Roseprincess`, `Redglobe`, and
`Redgold` to existing FruitFacts names to avoid normalized-name duplicates.

## NDSU FN590 Jams And Jellies Notes

NDSU FN590 is not primarily a horticulture publication, but its early sections
contain useful fruit cultivar grids for North Dakota gardeners. The tables are
not normal data tables with headers per cultivar row. Instead, most useful
cells are just cultivar names, and some cells are group labels or names with
type suffixes.

The config uses `html_name_matrix` for the strawberry, raspberry, apple, grape,
and Prunus grids. Raspberry rows inherit summer-bearing or fall-bearing
category context from group cells. Prunus rows map suffixes such as ` Plum`,
` Sour Cherry`, and ` Apricot` into plant types. Grape rows with parenthetical
regional caveats are normalized to clean cultivar names with source notes.

The Juneberry section names cultivars in prose rather than a table, so the
config uses fixed generated rows under `colon_paragraph_blocks`. The reference
keeps `needs_help` because the publication is a mixed food-preservation source
and does not provide complete cultivar detail for every fruit crop it discusses.

## UAF HGA-00030 Interior Alaska Notes

The UAF Interior Alaska variety list is a broad garden PDF with fruit rows
embedded near the end. The second fruit table page is regular enough for
layout-mode fixed-width slicing, but the first fruit page has row-spanned apple,
crabapple, cherry, currant, and gooseberry labels shifted against the wrong
rows by `pdftotext`.

The config uses `pdf_grouped_fixed_width_table` for the clean page. Group rules
carry fruit context from heading lines such as `Pear Note:` and from first-row
labels such as `Raspberry`. The resulting draft covers honeyberries, pears,
plums, raspberries, saskatoons, and strawberries, while keeping `needs_help`
for the shifted first page. Lee Red and Vic Red use explicit row overrides
because their descriptions share one extracted line.

## MSU MT202101AG Cold-Hardy Berries Notes

The Montana cold-hardy berry MontGuide is a two-column narrative PDF rather
than a table. The haskap cultivar paragraph can be parsed as quoted entries,
but Aurora and Borealis are described together. The config keeps that as a
declarative row split with a source note.

The dwarf sour cherry cultivar text is interleaved with pruning prose in the
left column. The config uses layout-mode text plus `line_slice_start` to parse
only the right-column cultivar prose, then applies small source-local
`row_text_fixes` for column-edge artifacts. Currant, gooseberry, and aronia
sections remain review caveats because they either point to other sources for
recommendations or lack a current FruitFacts plant type fit.

## UNH Low-Input Tree Fruits Notes

The UNH low-input tree fruit guide has a compact Table 2 in the PDF. Raw
`pdftotext` collapses that table into category headings followed by
comma-separated cultivar lists. Several headings appear before their lists,
and a few extracted lines contain two source categories at once.

The config uses `pdf_category_name_lists`. Simple category lines queue context
for the next cultivar list. Grouped category lines, such as `Sour Cherry
European Plum`, use an explicit `split_before` marker to separate the two
lists on the following extracted line. Footnote markers like `Reliance2,3` are
removed before comma splitting so they do not become false cultivar rows.

## SDSU P-00041-2023 South Dakota Notes

The SDSU fruit variety recommendations PDF has dense tables and two-column
pages. Some major tables, especially tree fruit and strawberries, are
transposed or split too aggressively by `pdftotext` for a broad first pass.

The config uses `pdf_catalog_entries` on bounded layout-mode slices for the
sections that remain structurally reviewable: currants, gooseberries,
raspberries, honeyberries, and dwarf tart cherries. Some slices start directly
on cultivar prose rather than a category heading, so the parser now supports
default category context. The Red Wing raspberry text contains the word
`Heritage.` before the true Heritage entry, so the right-column raspberry
section is split into two extractor passes around that source paragraph.

## KSRE MF1028 Kansas Notes

Kansas MF1028 is a dense two-column PDF with broad fruit tables. The full
publication covers tree fruit and small fruit, but the safest first pass is a
bounded layout-mode slice of the small-fruit table where the grape and
raspberry columns stay mechanically reproducible.

The config uses `pdf_catalog_entries` with `line_slice_start` for grapes and
`line_slice_end` for raspberries. Raspberry color labels such as Black, Yellow,
and Purple land on the same extracted lines as the first words of the next
cultivar note, so the config uses continuation skips plus row overrides for
Black Allen, Fall Gold, and Brandywine. The reference keeps `needs_help`
because the remaining stacked blackberry, blueberry, tree-fruit, and strawberry
tables need separate review.

## LSU Louisiana Home Orchard Notes

The LSU AgCenter home orchard PDF has a variety and spacing table that raw
`pdftotext` extracts as crop headings followed by wrapped cultivar-name lines.
Spacing values and explanatory notes are mixed into the cultivar stream, so a
simple comma splitter produces false names unless the heading and note rows are
handled explicitly.

The config uses `pdf_wrapped_name_lists`. Each heading sets the crop and
regional category, then the parser collects following lines until the next
heading. Source-local text fixes remove spacing-column artifacts such as `20`
inside peach and plum lists and repair wrapped names such as `Ichikikei Jiro`.
Repeated cultivars, such as peaches listed in two Louisiana regions, are merged
after record creation so one importable plant record keeps both source
descriptions. Mayhaw is intentionally left unencoded because the current plant
type list does not include it.

## VCE 422-017 Virginia Pear Notes

The VCE pear page uses quoted cultivar paragraphs under a European pear
heading, then switches to Asian pears with an ordinary prose paragraph rather
than a heading. The config uses `quoted_paragraph_blocks` with `stop_at` and
`start_after` prose selectors so the two pear types keep separate categories.

One extracted Asian pear paragraph contains both Kosui and Hosui. The config
keeps that source-specific cleanup as a declarative `row_splits` entry instead
of adding custom parser code.

## NMSU H-310 And H-326 Prose Notes

The NMSU orchard and minor-fruit guides name cultivars inside crop prose rather
than tables. The configs use `inline_quoted_names` with source-local
`names_after` and `names_before` slices for each recommendation sentence.

These sources showed two reusable quoted-name edge cases. Cultivar names can
contain internal apostrophes, such as `D'Anjou`, and a sliced phrase can end
immediately after the closing quote, such as the last cultivar in a list. The
shared quoted-name parser now handles both cases.

Alias names inside the same quoted list, such as `20th Century` and
`Nijisseiki`, should be handled with `exclude_names`, `AKA`, and source notes
so one plant record carries the source relationship without duplicating the
paragraph.

## Texas A&M EHT-017 Apple Notes

The Texas apple PDF lists the most useful cultivar recommendations in prose
before a table. The config uses two `pdf_marker_list` extractors bounded by the
`Varieties` and table markers to separate higher-chill and lower-chill apple
lists.

The source PDF wraps cultivar names in backtick and apostrophe quote pairs, and
one cultivar name contains an internal apostrophe. `pdf_marker_list` rows now
run through the normal cleanup pipeline, and `strip_name_quotes` removes only
the wrapping quote characters while leaving names such as `Mollie's Delicious`
intact.

## Texas A&M EHT-023 Pear Notes

The Texas pear PDF keeps its useful variety recommendations in raw prose lists
by zone. The config uses separate `pdf_marker_list` extractors for each zone
and pear type, with tight end markers so page numbers and parenthetical caveats
do not become cultivar names. Several pears are recommended in more than one
zone, so the config uses top-level duplicate merging and broad pear-type
categories while keeping the zone wording in each merged description.

## Arkansas FSA6129 Tree Fruit Notes

FSA6129 is a layout-mode PDF table where season and crop labels can share a
line with the first cultivar. Prefix category rules with `parse_remainder`
preserve the heading context while parsing the cultivar name that follows.
Trailing asterisks are kept as source notes for University of Arkansas
releases, and source spelling repairs are made through local name overrides.

## Cornell Fire Blight Apple Notes

The Cornell fire-blight page is a large HTML table, but citation superscripts
flatten into digits attached to rating labels. The config uses row regex fixes
to convert those repeated patterns into readable `source refs.` text before
the description is emitted.

## Rutgers FS1083 Plum Notes

Rutgers FS1083 is a two-column PDF. The config parses the left and right
columns separately with `line_slice_end` and `line_slice_start`, and the shared
catalog parser handles explicit entries followed by separators such as
`Name - description`. One Castleton page-break continuation is kept as a
source-local row override.

## UNH Small-Fruit Rating PDF Notes

The UNH blueberry and bramble factsheets are old migrated PDFs where the same
source table extracts differently depending on raw versus layout text.

For the bramble sheet, raw text preserves source order for some sections but
extracts names as one block and ratings/descriptions as a later block. The
`pdf_sequential_rating_rows` parser zips configured source names to rating
segments for the regular red and yellow raspberry sections. Blackberry rows
use `pdf_named_rating_rows`, which can hold a pending name when the source
puts the cultivar on one line and the rating payload on the next.

For the blueberry sheet, layout text keeps more row alignment, but some names
are still detached from their rating rows. The config extracts only rows that
can be tied back to visible names and leaves `needs_help` for the skipped
detached rows. This is preferable to guessing at the row offset.

## Arizona Catalog PDF Notes

UA AZ1162 and AZ1269 are catalog-like PDFs with useful `Name: description`
entries. Use raw text first for these guides: it preserves prose better than
layout text, even though it can place multiple cultivar entries on one line.

`pdf_catalog_entries` now has two useful switches for this shape:
`entry_names_only` prevents fallback parsing from turning continuation text
into fake cultivar names, and `split_embedded_entries` splits several embedded
`Name: description` entries out of one extracted line while attaching leading
continuation text to the previous row.

AZ1162 is safest when bounded by crop sections because `Early`, `Midseason`,
and `Late` headings repeat under several crops. AZ1269 needs row overrides for
known two-column heading drift, such as Asian pears, quince, persimmons,
almonds, and grapes appearing under neighboring headings. The kiwi rows were
skipped because the database does not currently define a `Kiwi` plant type.

## Illinois Routed Web Guide Notes

The Illinois Extension fruit-tree and small-fruit guides are navigation pages
with useful cultivar data spread across routed child pages. Configs can now set
`source` on individual extractors, so one reference config can keep the guide
landing page as its citation URL while fetching crop pages for the actual
records.

Use `inline_quoted_names` for child pages where source paragraphs say
`Suggested varieties include 'Name'...`. Keep tight `contains`, `names_after`,
and `names_before` rules because some paragraphs mention the same cultivars
again in pollination sentences. For region-specific duplicate cultivars, use a
broad category and merge duplicate plants while preserving the source regions
in descriptions.

The Illinois strawberry table keeps the cultivar group in the first column and
leaves following cells blank while preserving row width. The
`fill_down_first_column` table transform fills those group labels before
category mapping. This is different from row-spanning tables where following
rows are one cell short and `fill_leading_group_cells` is still the better
tool.
