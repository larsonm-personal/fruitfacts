# Source Asset And Stub Checklist

## Goal

Decide whether a candidate source deserves a downloaded asset, a JSON5 reference
stub, both, or neither.

## Default

For accepted durable sources with cultivar-level data, prefer both:

- Download the source asset for DVC tracking.
- Create a JSON5 reference stub with `needs_help: true`.

Use other outcomes only when the source shape makes that default awkward.

## Download And Stub

Choose this when most of these are true:

- The source has cultivar-level variety data.
- The source is from extension, a university, government, a trial program, a
  peer-reviewed publication, a book, or another durable authority.
- A PDF, report, spreadsheet, or other source file can be downloaded.
- The source is likely to support future encoding or source review.
- The source is not already represented by an equal or better existing
  reference.
- Metadata is good enough to make a useful citation.

Typical examples:

- Extension cultivar guides
- Trial reports
- Variety comparison bulletins
- Breeding program release summaries with downloadable PDFs
- Commercial charts that are useful as dated source snapshots

## Stub Only

Choose this when the source is worth citing but there is no useful downloaded
asset.

Good reasons:

- The source is a stable web article or database page.
- The source has no PDF or file download.
- The web page itself is the canonical source.
- The source is short enough that a downloaded snapshot adds little value.
- Downloading is blocked, fragile, or legally uncertain.

The stub should keep the best source URL and `needs_help: true` until encoding
or review is complete.

## Download Only

Use this sparingly, usually as a temporary queue state before stubbing.

Good reasons:

- The file may disappear and should be preserved before full triage finishes.
- The source needs human review before deciding whether it should become a
  FruitFacts reference.
- The file supports another reference as provenance but is not itself a plant
  data collection.
- The DVC workflow is being restored and the source should wait outside committed
  paths.

Do not let download-only sources become invisible. Record them in the candidate
queue with `asset_decision: "download only"` and a clear note.

## Neither

Choose this when the source does not currently deserve project storage or a
reference stub.

Common reasons:

- It has no cultivar-level data.
- It only repeats another source already captured.
- It is an uncited blog, sales page, summary, or AI-generated page.
- It requires account access or has unclear rights.
- It is off-topic for home gardeners or FruitFacts search goals.
- It is too vague to support citation or later recovery.

Keep the rejection in the candidate queue if rediscovery is likely.

## Revisit Later

Use this when the source might matter but the decision needs more context.

Examples:

- A crop or region gap is not yet prioritized.
- The source may be superseded by a newer edition.
- The source is valuable but needs archive or library access.
- The source is a citation trail rather than a final source.

## Quick Checklist

1. Does the source name specific cultivars or varieties?
2. Does it provide traits FruitFacts can use, such as harvest time, region,
   disease resistance, chill hours, trial location, patent status, or release
   history?
3. Is the source authoritative enough to cite?
4. Is it already covered by an existing reference?
5. Is there a downloadable source file?
6. Would losing the file harm future verification?
7. Is there enough metadata to recover or cite the source later?
8. Should the next action be `download and stub`, `stub only`, `download only`,
   `neither`, or `revisit later`?

## Open Questions

- Should `download only` ever lead directly to DVC tracking without a stub?
- Should the backend importer eventually report references without source
  assets?
