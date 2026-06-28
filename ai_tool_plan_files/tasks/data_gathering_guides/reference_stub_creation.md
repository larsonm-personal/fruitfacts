# Reference Stub Creation

## Goal

Create an initial JSON5 reference file so a downloaded or known source can be
tracked even before plant data is encoded.

## Inputs

- Source title, author, institution, URL, and publication dates
- Downloaded source file path when available
- Existing JSON5 examples in the target directory

## Output

A `.json5` reference file with metadata, source type, locations when known, an
empty or minimal plant list, and `needs_help: true`.

## Steps

1. Choose the target directory based on existing reference organization.
2. Copy the closest existing JSON5 shape rather than inventing a new one.
3. Fill citation metadata accurately.
4. Add known locations only when the source clearly supports them.
5. Set `needs_help: true` and add notes in fields that already support notes.

## Checks

- The file parses as JSON5.
- The source URL and title are enough to find the publication again.
- `needs_help` remains set until actual encoding work is complete.

## Open Questions

- Should stubs include placeholder categories or wait until encoding?
- What should be the minimum valid `plants` value for a stub?
