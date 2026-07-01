# js hints

- install nvm (there is a related windows project)
  - `nvm install lts` and `nvm use lts`
  - this sequence will also update to newer versions
- `npm run dev/build/start` next.js things
- `ncu -u` update package.json versions (after installing `npm i -g npm-check-updates`)
- `npm run lint` run next lint
- `npm run dev`
- `npm start`
- `.\start_local_test_server.ps1` run the local frontend server using repo-local Node/npm if needed
- `.\start_local_test_server.ps1 -Production` build and run the production-style local server
- `.\check_local_routes.ps1` build, start, and request local static, directory, and collection routes

# local dev setup

- alias `local.fruitfacts.xyz` to localhost and connect to `http://local.fruitfacts.xyz:3000` for OAuth-capable dev
- `local.fruitfacts.xyz` must resolve to localhost before the helper will start, so browser API calls and OAuth redirects use the same development host
- whitelist `http://local.fruitfacts.xyz:3000` as secure to allow geolocation - for chrome instructions see https://stackoverflow.com/a/55858436

# external issues I'm tracking

- `feb 2023`: getting cookies SSR is weird https://github.com/vercel/next.js/issues/45371
- `Jul 2022`: next.js: can't use getStaticProps and getServerSideProps together, so for example I can't generate the list of plant types for the search page's dropdown and then also make the search page server-side generated https://github.com/vercel/next.js/discussions/11424
- `Jan 2022`: next.js: not enough options for redirects so we can't have nested paths with some paths ending in '/' and some not
  - https://github.com/vercel/next.js/discussions/23988
- `May 2022`: react-zoom-thing has been abandoned for 10 months and hasn't gotten a react 18 update:
  - see overrides in package.json - remove these when possible
  - https://github.com/prc5/react-zoom-pan-pinch/issues/292
  - 2nd problem, it sets `fit-content` which makes zooming stuff be the wrong size. fixed with some global css to disable `fit-content`
  - https://github.com/prc5/react-zoom-pan-pinch/issues/112
