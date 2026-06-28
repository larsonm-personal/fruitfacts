# Existing Encoding Update

## Goal

Fill missing or incomplete data in an existing JSON5 reference from its source.

## Inputs

- Existing JSON5 reference file
- Matching source file or page
- Association pattern helper

## Output

An updated JSON5 file that preserves source meaning and clears `needs_help` only
when appropriate.

## Steps

1. Read the existing JSON5 file and nearby examples.
2. Compare the source against the encoded locations, categories, and plants.
3. Add missing cultivar-level facts without inventing precision.
4. Preserve the source wording for vague dates, chill descriptions, and notes.
5. Run a parser-backed or import-backed check when practical.

## Checks

- Existing field names and patterns are reused.
- Added data can be traced back to the source.
- `needs_help` is removed only when the known missing work is complete.

## Open Questions

- Which checks should be required before clearing `needs_help`?
- How should partial extraction from long sources be marked?
