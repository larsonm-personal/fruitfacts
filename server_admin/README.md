# Safe rebuild and deployment (Debian Linux)

Fruitfacts runs a Next.js Pages Router frontend (port 3000), an Actix/Rust API
(port 3001), SQLite at `backend/database.sqlite3`, and nginx with existing TLS
configuration. Rust embeds OAuth settings from `backend/.env` and `.env.release`
at compile time. Protect release directories and binaries as secrets.

`data_and_code_update.sh` now delegates to `scripts/deploy.sh`. **The old no-argument
reset/reload behavior is gone.** Neither script changes git history, imports data,
installs packages, changes ownership/security settings, edits nginx, runs certbot,
or downloads DVC media. Database schema migrations are not automated.

## Prerequisites

Review these on your server; this project does not run them automatically:

* Supported Debian, bash, git, curl, unzip, util-linux (`flock`), Python 3 (tests),
  SQLite CLI, `build-essential`, `pkg-config`, and `libssl-dev`. For example:
  `sudo apt-get install git curl unzip util-linux python3 sqlite3 build-essential pkg-config libssl-dev`.
* Node **24.22.1 or newer 24.x**, installed using a trusted installation method.
  `.nvmrc` selects 24.22.1. If using nvm: `nvm install && nvm use`.
  A systemd service does not load your interactive nvm setup: its absolute Node
  executable must match `command -v node` for the deploy user.
* Rust/rustup: `rustup show` in the checkout installs the pinned **1.99.1** toolchain.
  See https://rust-lang.org/tools/install/. No custom LLVM repository is needed.
* Enough disk for an entire new checkout, node_modules, Rust target directory and
  database backup for each release. Builds default to two workers; on a small VPS
  use `CARGO_BUILD_JOBS=1`. A 1 GB VPS may still need an off-server build or more RAM.
* A deployment account that can write the releases directory and read existing
  config/media, with narrowly authorized `sudo systemctl restart backend_fruitfacts
  frontend_fruitfacts`. Run builds as that account, not root. The service user must
  be able to traverse/read releases and write Next.js caches (use an existing shared
  group or deploy as the service account). This script does not manage permissions.

The existing **state checkout** (example `/var/www/fruitfacts`) must retain:

* `backend/.env`, `.env.release` and any customized `.env.debug`;
* `frontend/.env`, `.env.production`, and optional `.env.local`/`.env.production.local`;
* `backend/src/gazetteer_load/2021_Gaz_zcta_national.txt` (the real Census file);
* `frontend/public/data` with existing DVC media;
* for activation, the populated `backend/database.sqlite3` with user data.

The script copies configuration without replacing the originals, and shares only
media by symlink. SQLite stays in the original backend working directory (including
its WAL/SHM files). It never follows database reset/reload paths. Back up state,
configuration and certificates separately before the first service migration.

For a genuinely new installation, provision configuration and restore a populated
database first. `.env.example` contains placeholders, not usable OAuth credentials.
`./download_files.sh` retrieves the Census gazetteer without overwriting an existing
file. Run `dvc pull` separately with your existing authorized setup if media is needed.
An explicitly empty `frontend/public/data` is allowed for a development setup with
missing thumbnails. No credentials are generated. Initial database import using
`--reload_db` is an explicit data-management operation outside this deploy helper;
never use it casually against a database containing user data.

## Build from a new checkout

After checking out the update branch (or importing the supplied bundle):

```sh
cd fruitfacts
node --version                 # >=24.22.1 and <25
rustup show
./scripts/deploy.sh /var/www/fruitfacts /var/www/fruitfacts-releases
```

This builds committed HEAD in a fresh release, runs `npm ci`, Jest, Rust unit tests
and both production builds. Tracked local edits cause an early failure; commit
intentional source changes first. Untracked source files are not included. It does
not fetch code or stop running services. Failed and successful build directories
are retained for diagnosis. Concurrent invocations are rejected with `flock`.

## One-time service migration, then activation

Review the templates under `service_files/`; adapt absolute paths and the Node
executable for your installation. The helper verifies service working directories,
backend executable, and frontend Node executable before activation. It assumes the
standard local ports 3000 and 3001. The templates deliberately keep the backend
working directory at the existing state location.

Before replacing services, save their current definitions (`systemctl cat ...`) and
make the initial current symlink point at the old checkout, which must contain its
working `backend/harvest-chart-server`, frontend dependencies and build:

```sh
# As the owner of the prepared releases directory; do not replace an existing link.
ln -s /var/www/fruitfacts /var/www/fruitfacts-releases/current
# Review/adapt service templates, then explicitly install them as an administrator:
sudo install -m 644 server_admin/service_files/{backend,frontend}_fruitfacts.service /etc/systemd/system/
sudo systemctl daemon-reload
# Use the tested patched Node binary for both old and new frontend services.
./scripts/deploy.sh /var/www/fruitfacts /var/www/fruitfacts-releases --activate
```

Every activation first builds a fresh release while the previous one runs, checks
SQLite integrity and takes an online `.backup` (including committed WAL contents),
atomically switches `current`, restarts both services, then checks `/api/fact` and
`/plants` locally. There is a brief restart outage; this is not zero-downtime deployment.
Failed activation switches back and attempts to restart the old release. A failure
of that restart is printed and requires operator intervention. SIGKILL, power loss,
and external changes cannot be recovered by a shell trap. Inspect `current` and logs.

## Rollback and retention

A successful release contains `previous-release` and
`database-before-activation.sqlite3`. To roll back code, read and verify that path,
then perform the same atomic link switch and restart (serialize with deployment):

```sh
exec 9>/var/www/fruitfacts-releases/.deploy.lock
flock -n 9 || exit 1
previous=$(cat /var/www/fruitfacts-releases/current/previous-release)
test -d "$previous" || exit 1
ln -s "$previous" /var/www/fruitfacts-releases/rollback-next
mv -Tf /var/www/fruitfacts-releases/rollback-next /var/www/fruitfacts-releases/current
sudo systemctl restart backend_fruitfacts frontend_fruitfacts
curl --fail http://127.0.0.1:3000/plants
curl --fail http://127.0.0.1:3001/api/fact
```

Do not automatically restore the database backup: doing so would discard new user
writes. This update changes no migration SQL. Future schema changes need a separate
backup/migration/rollback plan. Review `journalctl -u backend_fruitfacts -u
frontend_fruitfacts` after activation or rollback. Retain at least the active and
previous release; manually remove only verified unused releases to reclaim disk.
Never remove the state checkout, shared media, configuration or certificate paths.

## Runtime choices and validation

Node 24 is LTS; 24.22.1 includes the October 2026 Brotli fix:
https://nodejs.org/en/blog/release/v24.22.1 . Next.js 16 is the supported Active LTS:
https://nextjs.org/support-policy . This update retains React 18.3.1/MUI 5 and
Tailwind 3 to limit UI migration scope; Next 16 supports React 18 but warns it will
be removed in Next 17. New React Compiler lint diagnostics remain warnings for
legacy effects; other lint rules remain enabled. The selector-parser override
patches a transitive build-tool denial-of-service advisory.

Run `python3 -m unittest discover -s scripts -p 'test_*.py' -v` for isolated deployment
control-flow tests. They stub builds, systemd, SQLite CLI and HTTP, and verify repeat
runs, build failures, health failure rollback, prerequisite failures and unchanged
state files; they are not a live systemd integration test. Real application builds
and tests must also pass. Run the existing ignored database import test only in a
disposable checkout/database: `cargo test --locked --no-default-features --lib --
--include-ignored` from backend. It resets data and can rewrite generated files.

No repository change establishes what Node version is installed on the actual VPS
or whether it is exposed to the advisory. No live server access is part of this work.

The backend uses OAuth2 5 / reqwest 0.12 and git2 0.21. Its unused HTTP/2
feature is disabled to exclude vulnerable h2 0.3; nginx uses HTTP/1 upstream.
OAuth token exchange uses a no-redirect client with a timeout. End-to-end Google
login still requires verification with the operator's existing configuration.
ESLint remains on 9.39.5 because current Next lint plugins reject ESLint 10 peers;
this is a development-tool support limitation. Cargo audit's unmaintained `paste`
warning comes from optional PDF tooling (excluded from server builds).

Release roots created by the helper use mode 0750 so an existing shared service
group can traverse them; the parent directory must already assign the intended
group (for example through an operator-managed setgid directory). The helper does
not change groups or ownership. Health checks require each service to be active.
