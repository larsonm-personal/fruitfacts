# Data Gathering And Reference Encoding

A primary goal of FruitFacts is to be an index for high-quality variety data
sources such as land grant university growing guides, books, published variety
comparison studies, trial reports, and other durable publications.

The long-term data gathering workflow should help with:

1. Finding new high-quality sources
2. Downloading and organizing source files
3. Creating initial JSON5 reference stubs with `needs_help: true`
4. Encoding useful plant data from sources into `plant_database/`
5. Clearing the `needs_help` backlog when missing work is understood
6. Sanity-checking source data, URLs, and encoded records

## Concrete Task Guides

These guides are intentionally concise placeholders. They should be expanded as
real work exposes repeated patterns, edge cases, and useful helper commands.

1. [Source Candidate Search](data_gathering_guides/source_candidate_search.md)
   - Search the web and AI search tools for new high-quality fruit variety data
     sources with concrete downloadable URLs
2. [Citation Survey](data_gathering_guides/citation_survey.md)
   - Mine existing references for citations, then locate the cited source files
3. [Candidate Triage And Prioritization](data_gathering_guides/candidate_triage.md)
   - Rank candidate sources before download or encoding work starts
4. [Source Download And Sorting](data_gathering_guides/source_download_and_sort.md)
   - Download source assets and place them in the same location/type pattern as
     existing references
5. [Reference Stub Creation](data_gathering_guides/reference_stub_creation.md)
   - Create initial `.json5` reference files with metadata and `needs_help`
6. [Association Review](data_gathering_guides/association_review.md)
   - Compare a source with its JSON5 file and update the shared association
     pattern helper
7. [Existing Encoding Update](data_gathering_guides/existing_encoding_update.md)
   - Fill missing plant data in an existing JSON5 reference from its source
8. [Greenfield Encoding](data_gathering_guides/greenfield_encoding.md)
   - Create a full JSON5 reference encoding from a source with no current file
9. [Help Needed Backlog](data_gathering_guides/help_needed_backlog.md)
   - Work through `needs_help` files and clear the marker only when justified
10. [Data Sanity Checks](data_gathering_guides/data_sanity_checks.md)
    - Check references and encoded data for structural and content problems
11. [Provenance And URL Verification](data_gathering_guides/provenance_url_verification.md)
    - Verify source identity, durable links, downloaded filenames, and metadata
12. [Reference URL Download Audit](data_gathering_guides/reference_url_download_audit.md)
    - Survey existing reference URLs for direct download success, landing-page
      download links, official replacements, and changed PDF hashes

## Shared Helper Docs

- [Association Patterns](data_gathering_guides/association_patterns.md)
  - A single living helper for how source rows, locations, categories, cultivar
    names, harvest times, and citations are mapped into FruitFacts JSON5
- [Candidate Source Queue Format](data_gathering_guides/candidate_source_queue.md)
  - JSON5 queue shape for source candidates before download, stubbing, or
    encoding
- [Source Asset And Stub Checklist](data_gathering_guides/source_asset_stub_checklist.md)
  - Decision helper for whether a source deserves a downloaded asset, a JSON5
    stub, both, or neither

## Working Notes

- Prefer source-preserving encodings over interpretation. For example, keep
  vague date wording vague unless the source itself gives exact dates.
- Prefer university extension, land grant, trial, peer-reviewed, book, and
  durable institutional sources over uncited editorial summaries.
- Preserve useful source metadata even if the plant data still needs later
  parsing.
- Use existing JSON5 field names and nearby examples before inventing new
  shapes.
- Do not clear `needs_help` unless the underlying missing or uncertain work has
  actually been resolved.

## Immediate Tasks

Completed setup passes:

1. Added concise placeholder guide docs for the current and expanded concrete
   tasks
2. Expanded the concrete task list to include triage, stub creation, backlog
   work, URL verification, and sanity checks
3. Added this file as the hub for all task guide links
4. Added a candidate-source JSON5 queue format
5. Added a checklist for deciding whether a source deserves a downloaded asset,
   a JSON5 stub, both, or neither
6. Added `ai_tool_plan_files/tasks/candidate_sources.example.json5` as the
   queue template
7. Added the initial `ai_tool_plan_files/tasks/candidate_sources.json5` queue

Useful next passes:

1. Fill `association_patterns.md` by reviewing 3 to 5 already-encoded
   references against their PDFs or source pages
2. Add helper commands once the DVC asset workflow is restored locally
