# Fruit Notes Workflow

Fruit Notes issues are mixed magazines: one issue may contain cultivar trials,
grower-practice articles, pest-management articles, obituaries, and ads. Treat
the issue as the inventory unit and each article as the potential FruitFacts
reference unit.

## Issue Index

Use `ai_tool_plan_files/tasks/fruit_notes_issue_index.json5` as the working
index.

Use `ai_tool_plan_files/tasks/fruit_notes_category_index.json5` as the shared
classification and parser-profile index. The issue index records what exists;
the category index records whether a family of articles should be encoded and
which parser shape to try first.

Record each issue with:

- `id`: `fruit-notes-v{volume}n{number}-{season-or-year}`
- `issue_url`: the full issue PDF when available
- `articles`: one object per article PDF, usually `a1.pdf`, `a2.pdf`, etc.

Useful article statuses:

- `new`
- `triage`
- `encoded`
- `encoded_needs_help`
- `do_not_encode`
- `revisit_later`

## URL And Filename Pattern

Most recent legacy Fruit Notes URLs follow:

- Issue PDF: `http://umassfruitnotes.com/v83n4/FN83-4.pdf`
- Article PDF: `http://www.umassfruitnotes.com/v83n4/a4.pdf`

Use one JSON5 reference per cultivar-rich article:

`plant_database/references/{State}/Fruit Notes- {Article Title}.json5`

Do not make one broad JSON5 file for a whole issue unless the issue itself is a
single coherent source.

Tag every encoded Fruit Notes article reference with `reference_categories`.
Use `fruit-notes` plus one issue label, for example:

```json5
reference_categories: ["fruit-notes", "fruit-notes-v85n2-spring-2020"],
```

## Encoding Rules

- Encode cultivar descriptions, variety trials, recommendation lists, and
  durable variety-performance updates.
- Skip pest, spray, subscription, obituary, and research-priority articles unless
  they contain cultivar-level observations useful to FruitFacts.
- Put the direct article PDF in `url`, not the full issue PDF, when the article
  PDF exists.
- Put issue metadata such as `Fruit Notes Volume 83, Number 4, Fall 2018` in
  `description`.
- Use the article's trial or farm location, not UMass Amherst, when the article
  describes cultivar observations from a specific site.
- Preserve source wording for relative or vague ripening phrases. Use
  `harvest_time_unparsed` for phrases like `late mid-season`; use
  `harvest_time_relative` only when the source gives an explicit relationship
  such as `1 week before Concord`.
- Leave `needs_help: true` when the article says a trial has more cultivars than
  it names, or when the import preserves only a partial source list.

## Local Extraction Commands

On this Windows checkout, Git for Windows provides `pdftotext.exe` even when
other Poppler commands are absent.

```powershell
New-Item -ItemType Directory -Force -Path tmp\pdfs | Out-Null
Invoke-WebRequest -UseBasicParsing -Uri http://www.umassfruitnotes.com/v83n4/a4.pdf -OutFile tmp\pdfs\a4.pdf
pdftotext -layout tmp\pdfs\a4.pdf tmp\pdfs\a4.txt
```

If DVC is unavailable, do not add downloaded PDFs directly to git. Keep direct
source URLs in JSON5 and note that the PDF asset was not archived in that pass.

## Narrative Profile PDFs

Some Fruit Notes cultivar-profile PDFs are born-digital two-column articles
where `pdftotext -layout` preserves page appearance but tangles prose. For
those, start with raw text and the `pdf_known_heading_entries` config parser.
Configure known cultivar headings, bound the article with `section_start` and
`section_end`, skip captions and page headers, and use reviewed `row_overrides`
for final descriptions.
