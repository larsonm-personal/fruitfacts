# Source Candidate Search

## Goal

Find new high-quality fruit variety data sources and record concrete
downloadable URLs for later triage.

## Inputs

- Existing `plant_database/references/` categories and naming patterns
- Search engines, AI search, university sites, extension sites, and archives
- Target crop, region, institution, or source type when known

## Output

A JSON5 queue entry following
[Candidate Source Queue Format](candidate_source_queue.md) with:

- Source title
- Institution or author
- Crop or crop group
- Region or trial location
- Downloadable URL
- Notes on likely value for FruitFacts

## Steps

1. Start from existing reference categories and search for similar sources.
2. Prefer durable institutional pages and PDFs over blog posts or summaries.
3. Record the exact downloadable URL, not just a search result page.
4. Note whether the source appears to contain cultivar-level data.
5. Send promising sources to candidate triage before download work.

## Checks

- The URL opens without requiring an account.
- The source names specific cultivars or varieties.
- The source has enough metadata to cite later.

## Open Questions

- Should low-priority sources be kept, discarded, or archived separately?
