# Help Needed Backlog

## Goal

Work through references marked `needs_help` and resolve the specific missing
work when possible.

## Inputs

- `plant_database/help_needed.md`
- Referenced JSON5 files
- Matching source files or pages

## Output

- Updated JSON5 files
- `needs_help` removed only for fully resolved references
- Notes for unresolved blockers

## Steps

1. Pick one `needs_help` reference.
2. Identify why help is needed before editing.
3. Compare the source and JSON5 file.
4. Make the smallest data update that resolves the known issue.
5. Keep `needs_help` if source access, OCR, ambiguity, or scale still blocks
   completion.

## Checks

- The reason for clearing `needs_help` is concrete.
- The import/database test still accepts the file.
- Generated `help_needed.md` is not edited by hand.

## Open Questions

- Should unresolved blockers be tracked in JSON5, a queue file, or both?
- Can `needs_help` reasons be standardized without making data files noisy?
