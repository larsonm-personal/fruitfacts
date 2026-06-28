# Provenance And URL Verification

## Goal

Verify that reference files point to the correct source and enough metadata is
present to recover that source later.

## Inputs

- JSON5 reference metadata
- Downloaded source assets
- Source URLs and landing pages

## Output

- Confirmed or corrected metadata
- Notes for dead links, moved files, or missing assets

## Steps

1. Open the stored URL and confirm it matches the reference title.
2. Check title, author, institution, publication date, and access date fields.
3. Confirm any downloaded asset is the same source as the JSON5 metadata.
4. Prefer durable landing pages plus direct download URLs when both are useful.
5. Mark unresolved source identity problems as needing help.

## Checks

- URL and downloaded file refer to the same publication.
- Metadata is not copied from a different edition without noting the difference.
- Dead links are not silently removed if they still explain provenance.

## Open Questions

- Should FruitFacts store both landing-page URLs and direct download URLs?
- How should archived URLs be represented?
