# Data Sanity Checks

## Goal

Find structural, source, and content problems in reference data before they
become user-facing errors.

## Inputs

- JSON5 reference files
- Backend import/database load tests
- Source files for suspicious records

## Output

- A list of suspected problems
- Fixes for clear issues
- Follow-up notes for uncertain issues

## Steps

1. Run parser-backed checks rather than ad hoc string checks when possible.
2. Look for missing source metadata, invalid plant types, duplicate names, and
   impossible dates.
3. Compare suspicious values with the original source.
4. Fix only cases that are clearly wrong.
5. Add new automated checks when the pattern is likely to recur.

## Checks

- Vague source dates are not converted into false precision.
- A failed check points to a source file and reason.
- Generated files are regenerated rather than edited by hand.

## Open Questions

- Which checks should live in Rust import tests?
- Which checks should be lightweight standalone scripts?
