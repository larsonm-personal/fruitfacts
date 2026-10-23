# js hints

- install nvm (there is a related windows project)
  - `nvm install lts` and `nvm use lts`
  - this sequence will also update to newer versions
- `npm run dev/build/start` next.js things
- `ncu -u` update package.json versions (after installing `npm i -g npm-check-updates`)
- `npm run lint` run next lint

# local dev setup

- alias `local.fruitfacts.xyz` to localhost. connect to `http://local.fruitfacts.xyz:3000` for dev - this allows using this url for oauth redirects
- whitelist `http://local.fruitfacts.xyz:3000` as secure to allow geolocation - for chrome instructions see https://stackoverflow.com/a/55858436

# external issues I'm tracking

- `Jul 2022`: next.js: can't use getStaticProps and getServerSideProps together, so for example I can't generate the list of plant types for the search page's dropdown and then also make the search page server-side generated https://github.com/vercel/next.js/discussions/11424
- `Jan 2022`: next.js: not enough options for redirects so we can't have nested paths with some paths ending in '/' and some not
  - https://github.com/vercel/next.js/discussions/23988
- `May 2022`: react-zoom-thing has been abandoned for 10 months and hasn't gotten a react 18 update:
  - see overrides in package.json - remove these when possible
  - https://github.com/prc5/react-zoom-pan-pinch/issues/292
  - 2nd problem, it sets `fit-content` which makes zooming stuff be the wrong size. fixed with some global css to disable `fit-content`
  - https://github.com/prc5/react-zoom-pan-pinch/issues/112

# Node security update: CVE-2026-111497

Use Node **22.23.4 or newer 22.x** for both builds and the running frontend.
From the repository root, `nvm install && nvm use` selects the tested version.
The existing system-wide installer now selects Node 22; do not rerun that entire
machine setup script merely to update Node. Update your existing system-wide Node
installation through its package manager, then check `sudo -u www-data node -v`.
The existing service still runs `npm start`; its new `prestart` check refuses an
unpatched Node version. `prebuild` enforces the same requirement. Restart the
frontend after installing the patched runtime; an already running process retains
the old runtime. Check `systemctl status frontend_fruitfacts` and its journal to
confirm the printed Node version and successful start. If the service has a custom
PATH, verify that environment too. An nvm upgrade in your shell alone does not
update systemd's Node.

The original `data_and_code_update.sh` full SQLite rebuild/import, maintenance page,
DVC and service workflow is unchanged. This PR does not change Next.js, React, database import logic,
database behavior, certificates or server security settings. It does not establish
which runtime is currently installed/exposed on the VPS. Cargo.lock and
package-lock.json are intentionally untracked; CI resolves
dependencies from the manifests with `cargo build` and `npm install`.

Advisory: https://nodejs.org/en/blog/vulnerability/october-2026-security-releases
