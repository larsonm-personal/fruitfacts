# Candidate Triage And Prioritization

## Goal

Decide which candidate sources deserve download, stubbing, encoding, or no
action.

## Inputs

- Candidate source list
- Existing FruitFacts reference coverage
- Current data gaps by crop, region, or trait

## Output

Each candidate gets a status:

- Download and stub
- Stub only
- Revisit later
- Reject

## Steps

1. Check whether FruitFacts already has the source or a newer equivalent.
2. Prefer sources with cultivar-level data, locations, and harvest timing.
3. Prefer sources from extension, trial, university, government, or published
   references.
4. Note the main reason for accepting or rejecting the candidate.
5. Send accepted candidates to download and sorting.

## Checks

- The candidate is not a duplicate of an existing reference.
- The source has enough metadata for a useful citation.
- The expected data value is clear enough to justify later encoding time.

## Open Questions

- What priority labels should be used?
- Should regional gaps outweigh crop gaps or vice versa?
