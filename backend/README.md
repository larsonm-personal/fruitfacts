# rust hints
* copy the .env.example file and fill in secrets as needed
* `cargo build`
* `cargo run`
* `cargo test` and `cargo test -- --include-ignored` (include long tests: the json database loading one)
* `cargo fetch` install packages

# support tools
* `cargo run --bin pdf_to_thumbnail` create thumbnails for each reference pdf. needs pdfium downloaded and installed, see instructions at https://crates.io/crates/pdfium-render
  * redo all thumbnails: `cargo run --bin pdf_to_thumbnail -- --redo_all`
  * tldr: on windows, into `/backend/`, place the dll from `pdfium-win-x64.tgz` from https://github.com/bblanchon/pdfium-binaries/releases
* `cargo run --bin web_thumbnails` create thumbnails for references tagged with "thumbnail: website"
* `cargo run --bin delete_extra_thumbnails` looks in /frontend/public/data/ for any thumbnail files without a corresponding *.json5 file in /references/ and deletes them. useful if things are renamed

# rust linting
* `cargo fmt` after installing `rustup component add rustfmt`
* `cargo fix`
* `cargo clippy` after installing `rustup component add clippy`
  * `cargo clippy --fix`
* `cargo outdated -d1` find outdated packages (-d1: direct only) or `cargo outdated` (all) after installing `cargo install --locked cargo-outdated` (same command to update)
* `cargo +nightly udeps` to find unused dependencies, after installing (`rustup toolchain install nightly` then `cargo install cargo-udeps --locked`) see https://crates.io/crates/cargo-udeps
* `cargo tree --duplicates` find dependencies with multiple required versions

# after a rust version release
* see `rust_update.bat`:
  * `rustup update stable`
  * `rustup toolchain install nightly`
  * `cargo install --locked cargo-outdated`
  * `cargo install cargo-udeps --locked`
  * `cargo test -- --include-ignored`

# oauth account stuff
* google https://console.cloud.google.com/apis/dashboard?pli=1 click credentials, click the edit pen, add an "authorized redirect URI" like http://domain.com/api/authRedirect

# debugging in vs code
* see extensions.json for recommended extensions
  * 2021: on windows vs code, `codelldb+rust-analyzer` debugger works slightly better than cppvsdbg or the official "rust" extension. see launch.json

# diesel (rust ORM) things
* see https://diesel.rs/guides/getting-started
* Use a current Diesel 2.3 CLI with bundled SQLite for development; the application runs embedded migrations and does not require the CLI on the server. See the [official CLI installation options](https://diesel.rs/guides/getting-started.html).
* adding a new table with diesel_cli:
* `diesel migration generate [new table name]`
* `diesel migration run` - or omit this and just run all tests, there are embedded migrations
* `diesel migration redo` (checks up+down)

# external issues I'm tracking
* support loading sqlite modules in diesel in order to use spatialite
  * https://github.com/diesel-rs/diesel/issues/1867
  * https://github.com/diesel-rs/diesel/pull/2180
* rust cargo: use lld on windows for faster builds. will eventually be default and I can remove the /.cargo/config.toml entry
  * in my testing this doesn't improve full build time at all
  * https://github.com/rust-lang/rust/issues/71520
* rust cargo: why are things rebuilt on the server? spurious rebuilds are a very common problem
  * https://github.com/rust-lang/cargo/issues/2904

# dependency modernization (October 2026)

Registry dependencies are exact-pinned to the stable releases reported by the
[crates.io API](https://crates.io/data-access) on 2026-10-23; see `Cargo.toml` for
versions. `dotenv-build` 0.1.1 is still its latest stable release. The custom
`expiring_map` fork remains necessary for its removal API and is pinned to the
existing branch tip by commit. Cargo.lock remains ignored by project policy.
Use current stable Rust (validated locally with 1.99.1); dependencies can require
newer Rust as their allowed transitive versions evolve.

OAuth 5 uses its typed endpoint builder and HTTP client interface. Its own
Reqwest 0.12 adapter remains a transitive dependency; application HTTP requests
use the latest direct Reqwest 0.13. Redirects are disabled for OAuth HTTP calls,
state is checked before token exchange, and token/session values are not logged
by those migrated paths. Tests cover state rejection, verifier consumption and
PKCE URL generation; a real Google login still needs deployment credentials.

PDF thumbnails use PDFium 0.9's fallible image conversion. Install a Pdfium shared
library compatible with the crate's default API version before running that
optional tool; an old DLL/shared library may be missing required symbols. The
production server uses `--no-default-features` and does not load Pdfium. See the
[upstream binding documentation](https://docs.rs/pdfium-render/latest/pdfium_render/).

For reference status and the existing server rebuild workflow, see
[REFERENCE_STATUS.md](../server_admin/REFERENCE_STATUS.md).
