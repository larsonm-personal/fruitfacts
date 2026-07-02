# Citation Survey

## Goal

Use citations inside existing references to find additional primary or
high-value sources.

## Inputs

- Existing JSON5 reference files
- Downloaded PDFs or source pages
- Bibliographies, footnotes, tables, captions, and acknowledgements

## Output

A candidate source list with located URLs where possible.

The first lightweight mined output lives here:

`ai_tool_plan_files/tasks/reference_mined_candidate_strings.json5`

This file is intentionally only an array of strings. Each string keeps the
source PDF, cited title or citation text, any scanned URL, the scanner match
kind, and short triage notes.

## Steps

1. Pick a completed reference or a high-value `needs_help` reference.
2. Extract cited reports, bulletins, trial papers, and variety lists.
3. Search for each citation by exact title first.
4. Record the best downloadable URL and fallback landing page.
5. Add unresolved citations to a later lookup list rather than guessing.

## Helper Script

Run the scanner across local reference PDFs:

```powershell
python helper_scripts/scan_reference_citations.py --limit 120 --min-score 90
```

Write a JSON array to a file:

```powershell
python helper_scripts/scan_reference_citations.py --limit 120 --min-score 90 --output $env:TEMP\fruitfacts_reference_scan_candidates.json
```

Write a reviewed aggregate array back into each source reference as
`found_citations`:

```powershell
python helper_scripts/scan_reference_citations.py --input ai_tool_plan_files/tasks/reference_mined_candidate_strings.json5 --write-reference-json5
```

Scan one PDF:

```powershell
python helper_scripts/scan_reference_citations.py --pdf "plant_database/references/Oregon/2017 - Table Grape Cultivar Performance in Oregon's Willamette Valley.pdf"
```

The scanner uses local `pdftotext` output for visible reference sections and a
byte-level URL scan for embedded links. It looks for common headings such as
`References`, `Literature Cited`, `Additional Resources`, and `For more
information`, then emits scored strings rather than structured candidates. The
scores are only a sorting aid. Human review still decides whether a citation is
a real source candidate, already covered, or too cultural/management-focused
for FruitFacts.

Reviewed citation strings can be kept in two places:

- `reference_mined_candidate_strings.json5`, as the aggregate review queue
- each source reference's top-level `found_citations` array, so the citations
  stay next to the source where they were found

`found_citations` is source metadata for future mining and is not currently
used for display.

Current limitations:

- PDF text can merge several citations into one long string.
- Wrapped URLs may still need manual repair.
- Institutional homepages and pest-management sources can score highly when
  they appear near fruit/cultivar terms.
- The scanner does not verify that cited URLs still resolve.

## Checks

- The located source title matches the cited title closely.
- The source date, author, and institution match when available.
- The citation is likely to add cultivar-level data not already captured.

## Open Questions

- Should citations be stored in a separate queue from web-search candidates?
- How should unavailable but important citations be tracked?
