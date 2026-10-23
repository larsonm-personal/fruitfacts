# Reference work and rebuilding the existing server

## What is and is not on main

Inspected on 2026-10-23. PR #1 was merged at
`7fbd65660c2ac17f51c8baf7c3b91f4cac2c8885`. The AI-assisted reference work is
on the separate public [`2026` branch](https://github.com/larsonm-personal/fruitfacts/tree/dbca4bfd30f3077fb2912b930561113d80c505be),
inspected at `dbca4bfd30f3077fb2912b930561113d80c505be` (last commit July 4 UTC).
Its merge base with main is `acceef27093e8b4cfa4a3cd4ad7d0a37d5d3caed`, and
173 commits on that branch are not ancestors of main. This modernization PR
does not merge or copy that branch. Rebuilding main will **not** publish those
additional references.

Counts below come from parsing tracked reference JSON5, not from the live site.
Plant rows are source records, not unique varieties or final database row counts.
`plant_database/help_needed.md` is a generated snapshot and is stale on the
`2026` branch (for example, it omits the Kansas MF1028 `needs_help` reference).
The counts here use JSON5 fields directly, not that generated list.

| Tracked content | Main after PR #1 | `2026` branch |
| --- | ---: | ---: |
| Reference JSON5 files | 254 | 385 |
| Source plant rows | 10,906 | 14,946 |
| References with `needs_help: true` | 143 | 250 |
| References with empty plant lists | 105 | 108 |
| Reference PDF DVC pointers | 139 | 190 |
| Frontend data/thumbnail DVC pointers | 213 | 255 |

The branch adds 131 reference files and changes 22 existing reference files.
It also adds an extraction helper library and 122 manifest entries in
`helper_scripts/extraction_manifest.json`. The guides explicitly describe
extractor output as a draft requiring curation, not automatically reviewed data.
The candidate queue contains 382 entries: 122 `encoded`, 9 `accepted`, and 251
`new`. Every encoded candidate with a proposed reference path has that file in
the branch. `encoded` does not necessarily mean complete: the Kansas MF1028
reference explicitly describes its partial coverage and retains `needs_help`.

The Fruit Notes issue index contains 17 issues (16 `partially_encoded`, one
`triaged`) and 107 article records: 24 `encoded`, 72 `do_not_encode`, one
`queued`, one `index_only`, six `triage`, and three `new`. Notes record some
source review, including visual review of the 1939 peach ripening graph, but
these are project notes, not an independently verified approval record. The
`reviewed` fields in references often describe the publication's own review
date; they must not be counted as human approval of AI output.

No local unpublished drafts, live server database, DVC remote object availability,
or end-to-end source accuracy was verified. The branch's `tmp/` directory is
ignored, and extraction helpers can emit drafts to stdout or a chosen directory;
those uncommitted outputs cannot be inferred from Git. A DVC pointer proves
that an asset was recorded, not that it is still downloadable. No DVC credentials
were accessed and no private source assets were downloaded for this review.

## What reaches the application

`backend/src/import_db.rs` reads JSON5 under `plant_database/references/` on the
checked-out revision. It imports files regardless of `needs_help`; that flag
and publication metadata are stored for display. Empty stubs contribute
reference metadata but no plant rows. Extractor queues and generated drafts
outside that directory are not imported. The frontend reads the resulting API
and SQLite data; rebuilding the JavaScript alone does not import references.

The full `--reload_db` operation clears and recomputes reference-derived tables,
relative harvest calculations, FTS, and generated summaries. The existing code
intentionally retains users, user-owned locations, and user-owned collection
items. It is not an atomic deployment or an automatic backup mechanism.

The `2026` branch also changes importer/API/UI code and adds a
`2026-07-03-000001_collection_categories` migration; 24 reference files use
`reference_categories`. It needs a separate reviewed integration with current
main and an import test before deployment. Copying only its data would omit
category behavior. Neither this PR nor the commands below perform that merge.

## Rebuild main on an already configured server

These instructions use the existing `/var/www/fruitfacts` installation and its
`backend_fruitfacts` / `frontend_fruitfacts` systemd services. They are not a
first-install recipe. Do not rerun the old Ubuntu provisioning script on a
Debian server: it includes Ubuntu-specific package sources and certificate setup.

Before scheduling downtime:

- Merge the desired code first, then use a clean checkout of the intended branch
  tracking `larsonm-personal/fruitfacts`. Inspect `git remote -v`, `git branch -vv`,
  and `git status --short`; save local edits elsewhere. The existing rebuild
  script runs `git checkout .` and would discard tracked edits.
- Install the Node version in `.nvmrc` for the service user as well as your shell,
  and a Rust toolchain meeting `backend/Cargo.toml`'s requirements. Check
  `sudo -u www-data node -v`, `rustc --version`, and `cargo --version` in the
  administrative shell that will run the rebuild. Compiler/linker prerequisites
  include build tools, pkg-config, OpenSSL development files, clang and lld.
- Preserve existing `backend/.env` (OAuth client ID/secret and cookie domain),
  profile environment files, any frontend configuration, DVC credentials, and
  nginx/certificate configuration. Backend environment values are compiled into
  the binary. Never replace real values with `.env.example` during an update.
- DVC with Google Drive support and the existing service-account credential file
  at the path configured in `.dvc/config` are required. Run `dvc pull` as the
  same administrative user that will run the script to validate access before
  downtime. Source PDFs and thumbnails are restored through DVC, not Git alone.
- Ensure the real Census gazetteer exists at
  `backend/src/gazetteer_load/2021_Gaz_zcta_national.txt`; if missing, run
  `./download_files.sh` (requires wget and unzip). Do not deploy a test fixture.
- Keep full Git history: the importer uses historical commit timestamps.
  Keep sufficient disk/RAM for frontend and Rust production builds. The old
  one-GB-server assumptions have not been validated for current dependencies.

From an administrative shell with the required toolchain on PATH, after those
checks and with no concurrent deployment running:

```sh
set -eu
cd /var/www/fruitfacts
backup_dir="/var/backups/fruitfacts/$(date -u +%Y%m%dT%H%M%SZ)"
install -d -m 700 "$backup_dir"
git rev-parse HEAD > "$backup_dir/revision-before-update"
sudo -u www-data git pull --ff-only
service backend_fruitfacts stop
sqlite3 backend/database.sqlite3 ".backup '$backup_dir/database.sqlite3'"
cp -a backend/.env "$backup_dir/backend.env"
# Also retain existing configuration and a known-good code/binary backup securely.
bash ./data_and_code_update.sh
```

Run the backup and rebuild from a shell authorized to stop services and write
`/var/backups`; do not give credentials to the script as command-line arguments.
The SQLite CLI must be installed for the backup command. Verify the backup
succeeded before continuing. Record the previously deployed revision separately
if the checkout has already advanced beyond it. Keep backups outside the checkout
and do not commit them. The script leaves `journalctl -xefu backend_fruitfacts`
running at the end; Ctrl-C exits that log viewer, not the services.

The existing script pulls the selected branch and DVC assets, installs the
maintenance page, stops both services, installs/builds the frontend, runs
`cargo run --release --no-default-features -- --reload_db`, copies the backend
binary, starts services, and removes maintenance mode. It regenerates local
lockfiles as a normal side effect; neither lockfile is committed. PDFs/thumbnails
are not regenerated by this server script.

Afterwards check `systemctl status backend_fruitfacts frontend_fruitfacts`, the
service journals, the homepage version/source link, and a known reference and
search result. The homepage revision identifies built code, not proof that the
unmerged `2026` data was incorporated.

If a build/import fails, `set -e` can leave maintenance mode and stopped services
in place. Do not delete the database or certificates to retry. Retain the error
log, stop both services before recovery, restore the known-good code and build
artifacts plus the consistent database backup, preserve the existing secrets,
and start/verify both services before removing the maintenance page. Do not copy
a database over a running SQLite process. Restoring the backup also discards
user writes made after that backup; keeping the backend stopped during this
sequence avoids that gap. Rollback and the live server itself were not exercised
in this PR.
