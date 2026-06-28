# Source Download And Sorting

## Goal

Download accepted source assets and place them in a predictable location for
later DVC tracking and JSON5 encoding.

## Inputs

- Accepted candidate sources
- Downloadable URLs
- Existing `plant_database/references/` directory patterns
- DVC asset conventions once restored

## Output

- Downloaded source file
- Proposed reference path and filename
- Notes on whether a JSON5 stub exists

## Steps

1. Download from the most durable URL available.
2. Preserve the original file extension.
3. Name files clearly using institution, crop, region, or source title.
4. Sort by state, region, institution, commercial source, or crop type following
   nearby examples.
5. Add or update DVC pointers only through the established asset workflow.

## Checks

- The downloaded file opens locally.
- The filename is readable and specific.
- The file is not committed directly if it should be DVC-managed.

## Open Questions

- Where should temporary downloaded candidates live before DVC tracking?
- Should source filenames match JSON5 filenames exactly?
