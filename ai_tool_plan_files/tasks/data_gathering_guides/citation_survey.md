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

## Steps

1. Pick a completed reference or a high-value `needs_help` reference.
2. Extract cited reports, bulletins, trial papers, and variety lists.
3. Search for each citation by exact title first.
4. Record the best downloadable URL and fallback landing page.
5. Add unresolved citations to a later lookup list rather than guessing.

## Checks

- The located source title matches the cited title closely.
- The source date, author, and institution match when available.
- The citation is likely to add cultivar-level data not already captured.

## Open Questions

- Should citations be stored in a separate queue from web-search candidates?
- How should unavailable but important citations be tracked?
