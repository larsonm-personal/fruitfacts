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

Use Node **24.22.1 or newer 24.x** for both builds and the running frontend.
From the repository root, `nvm install && nvm use` selects the tested version.
The existing system-wide installer now selects Node 24; do not rerun that entire
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
DVC and service workflow is unchanged. Cargo.lock and package-lock.json are intentionally
untracked; CI resolves dependencies from the manifests with `cargo build` and `npm install`.

## Frontend development

Use `npm install`, `npm test`, `npm run lint`, and `npm run build` with the Node version
in `.nvmrc`. `npm start` serves the production build. Next 16 uses the existing webpack
configuration explicitly; the app retains its Pages Router and existing URL structure.
Tailwind 4 uses `@tailwindcss/postcss` and the existing theme configuration. Its browser
baseline is Safari 16.4+, Chrome 111+, and Firefox 128+.

Direct versions were checked against the npm registry on 2026-10-23. Next 16.4.3,
React 19.3.1, MUI 9.5.0 and Tailwind 4.3.3 are the current stable releases. ESLint is
on the maintained 9.39.5 line because the current Next lint-plugin graph does not yet
support ESLint 10 (a fresh `npm ls` reports invalid peers with 10.13.0). No peer overrides
are used. Unused OpenLayers, next-transpile-modules and fs-extra were removed;
React-18-only react-debounce-input was replaced by a small tested input component.
Source links always point to larsonm-personal/fruitfacts, even with stale GitHub URL
overrides in a server's environment.

Migration references: [Next 16](https://nextjs.org/docs/app/guides/upgrading/version-16),
[Tailwind 4](https://tailwindcss.com/docs/upgrade-guide), and
[Node release policy](https://nodejs.org/en/about/previous-releases).

Advisory: https://nodejs.org/en/blog/vulnerability/october-2026-security-releases

## Local checks

After `npm install`, run `npm run check` for strict ESLint (zero warnings), Prettier
checking, tests, and the production build. `node_lint.bat` runs the same sequence
on Windows. Unlike the older 2026 helper, this never deletes dependencies, upgrades
versions, or forces installation. Use `npm run format` to apply formatting.

For the route sweep adapted from `check_local_routes.ps1`, start a local backend
with an imported test database, build the frontend against its API URL, and start
that frontend on a loopback port. Then run:

```sh
npm run check:routes -- http://127.0.0.1:3030
```

This checks the homepage, plants, map, search, and all reference/directory routes
from the checked-out JSON5 filenames. A third argument limits the route count,
for example `npm run check:routes -- http://127.0.0.1:3030 4`. HTTP errors and
backend errors reported in the server-rendered page data fail the check. It does
not start/stop services, rebuild the database, fetch DVC assets, or access a live
server. Browser interactions and external map tiles require separate testing.
