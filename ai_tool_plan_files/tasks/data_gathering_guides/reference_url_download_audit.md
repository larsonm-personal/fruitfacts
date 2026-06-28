# Reference URL Download Audit

## Goal

Survey existing reference URLs to find which sources still download, which
source pages still expose a download link, and which moved sources can be
updated to a current official reference.

## Inputs

- JSON5 reference files with `url` fields
- Existing `.pdf.dvc` pointer files when present
- Temporary download directory outside committed source paths
- Current source pages and institutional search results

## Output

- An audit list with one row per reference URL
- Status for direct URL fetch success, landing-page success, and download-link
  discovery
- Candidate replacement URLs for moved official sources
- Hash comparison results for newly found PDF files
- Follow-up flags for content review, metadata update, or DVC update

## Steps

1. Extract existing reference URLs from JSON5 files.
2. For each URL, classify it as a direct file URL, landing page, web article, or
   unknown shape.
3. Attempt a normal fetch and record status code, final URL, content type, and
   whether the body appears to match the expected title.
4. For landing pages or web articles, look for official PDF or download links on
   the page before searching the broader web.
5. If the original URL fails, search official institution domains for the same
   title, publication number, author, or filename.
6. Download candidate PDFs only to a temporary directory.
7. Compute the candidate file hash and compare it with the existing DVC `md5`
   when a pointer file exists.
8. Flag changed hashes for content review before replacing the source file or
   updating DVC pointers.
9. Record unresolved failures rather than silently removing old provenance.

## Checks

- A successful landing page is not treated as a successful file download unless
  it exposes a clear file link or intentionally is the source.
- Replacement URLs stay on official institutional, publisher, archive, or
  otherwise credible domains.
- A different PDF hash is flagged even when the title and publication number
  appear unchanged.
- Hash differences are not assumed bad. They may indicate a revised edition,
  regenerated PDF, changed cover page, accessibility update, or a different
  source.
- DVC pointer files are not updated until the changed file has been reviewed.

## Suggested Audit Fields

- `reference_path`
- `stored_url`
- `stored_url_status`
- `final_url`
- `content_type`
- `direct_download_ok`
- `landing_page_ok`
- `download_url_found`
- `replacement_url`
- `old_dvc_md5`
- `new_pdf_md5`
- `hash_changed`
- `needs_content_review`
- `notes`

## Open Questions

- Should the audit output live as JSON5, CSV, or a generated markdown report?
- Should this become a backend helper, a PowerShell helper, or both?
- How should multiple official replacement files be ranked?
