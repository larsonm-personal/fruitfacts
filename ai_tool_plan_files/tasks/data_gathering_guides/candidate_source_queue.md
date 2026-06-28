# Candidate Source Queue Format

## Goal

Use a small JSON5 queue for source candidates that are not ready to become
reference files yet.

JSON5 is preferred over a spreadsheet because it is versionable, commentable,
and already matches the project data style.

## Queue Location

The starter queue lives here:

`ai_tool_plan_files/tasks/candidate_sources.json5`

Keep rejected or low-priority candidates in the same file until the list becomes
large enough to split.

## Top-Level Shape

```json5
{
    notes: [
        "Candidate sources found before download, stubbing, or encoding"
    ],
    candidates: [
        {
            id: "uf-blueberry-guide-2024",
            status: "new",
            priority: "medium",
            title: "Example fruit variety guide",
            institution: "Example University Extension",
            authors: ["First Last"],
            region: "Florida",
            crops: ["Blueberry"],
            source_type: "state extension guide",
            landing_url: "https://example.edu/example-guide",
            download_url: "https://example.edu/example-guide.pdf",
            discovered: "2026-06-28",
            discovered_by: "manual search",
            value_summary: "Cultivar table with region notes and harvest time",
            asset_decision: "download and stub",
            proposed_reference_path:
                "plant_database/references/Florida/Example Guide.json5",
            duplicate_check: "No obvious existing reference",
            notes: [
                "PDF link found on the landing page"
            ]
        }
    ]
}
```

## Required Fields

- `id`
- `status`
- `title`
- `institution` or `authors`
- `region`
- `crops`
- `landing_url` or `download_url`
- `discovered`
- `value_summary`
- `asset_decision`

## Useful Optional Fields

- `publication_number`
- `published`
- `reviewed`
- `accessed`
- `source_type`
- `trial_locations`
- `candidate_filename`
- `proposed_reference_path`
- `duplicate_check`
- `archive_url`
- `notes`

## Status Values

- `new`
- `triage`
- `accepted`
- `downloaded`
- `stubbed`
- `encoded`
- `revisit_later`
- `rejected`

## Priority Values

- `high`
- `medium`
- `low`

## Asset Decision Values

- `download and stub`
- `download only`
- `stub only`
- `neither`
- `revisit later`

## Rules

- Keep one candidate per source, not one per URL.
- Prefer the official landing page in `landing_url`.
- Prefer a direct PDF or file link in `download_url` when one exists.
- Do not replace a candidate URL with a newer URL until the old one is recorded
  in `notes` or `archive_url`.
- Keep rejection notes. A rejected source found twice is still useful knowledge.
- Promote accepted candidates to real JSON5 reference files only after triage.

## Open Questions

- Should accepted candidates be removed from the queue or marked `stubbed`?
- Should this queue eventually be generated from a web audit report?
