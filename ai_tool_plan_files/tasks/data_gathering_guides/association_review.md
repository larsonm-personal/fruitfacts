# Association Review

## Goal

Compare a source with an existing JSON5 encoding and document recurring mapping
patterns in the shared association helper.

## Inputs

- Source PDF, page, image, or table
- Matching JSON5 reference file
- Existing [Association Patterns](association_patterns.md)

## Output

- Notes added to `association_patterns.md`
- Optional issue notes for data that looks missing or ambiguous

## Steps

1. Identify how the source organizes locations, crops, categories, and rows.
2. Compare that structure with the JSON5 fields used in the existing encoding.
3. Note useful mapping patterns with short examples.
4. Note ambiguous or inconsistent patterns separately.
5. Avoid changing data unless the review clearly reveals a small correction.

## Checks

- The helper doc describes patterns generally, not one-off source trivia.
- Examples use existing field names.
- Unresolved ambiguity remains marked rather than silently normalized.

## Open Questions

- Which associations are common enough to deserve canonical examples?
- Should the helper include a glossary for source-table phrases?
