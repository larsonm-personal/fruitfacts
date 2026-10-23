# js hints

- install nvm (there is a related windows project)
  - `nvm install` and `nvm use` from the repository root (.nvmrc)
  - this sequence will also update to newer versions
- `npm run dev/build/start` next.js things
- `ncu -u` update package.json versions (after installing `npm i -g npm-check-updates`)
- `npm run lint` run ESLint

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
