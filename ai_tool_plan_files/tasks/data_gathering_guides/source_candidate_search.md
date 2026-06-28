# Source Candidate Search

## Goal

Find new high-quality fruit variety data sources and record concrete
downloadable URLs for later triage.

## Inputs

- Existing `plant_database/references/` categories and naming patterns
- Search engines, AI search, university sites, extension sites, and archives
- Target crop, region, institution, or source type when known

## Output

A JSON5 queue entry following
[Candidate Source Queue Format](candidate_source_queue.md) with:

- Source title
- Institution or author
- Crop or crop group
- Region or trial location
- Downloadable URL
- Notes on likely value for FruitFacts

## Steps

1. Check existing `plant_database/references/` entries first so candidate work does
   not duplicate an already-encoded source.
2. Start from existing reference categories and search for similar sources.
3. Prefer durable institutional pages and PDFs over blog posts or summaries.
4. Record the landing page and exact downloadable URL when both exist.
5. Note whether the source appears to contain cultivar-level data.
6. Send promising sources to candidate triage before download work.

## Search Strategy

Start narrow, then broaden only when the narrow search fails. The best searches
so far have combined one or more of:

- An exact title fragment in quotes
- A publication number
- An institution-specific `site:` filter
- The word `extension`, `publication`, `bulletin`, or `cultivar`

Broad searches such as `best fruit trees for Kansas` find useful sources only
after a lot of pruning. They are best used for clue-finding, not for queueing a
source directly.

## Query Recipes

Use these patterns when searching for new candidates:

- `site:bookstore.ksre.ksu.edu cultivar fruit Kansas MF1028`
- `site:extensionpubs.unl.edu "Fruit Tree Cultivars" Nebraska "G2354"`
- `site:shop.iastate.edu "PM453" "Fruit Cultivars"`
- `site:extension.umd.edu "EB-2023-0684" "Important Apple Cultivars"`
- `"Fruit Cultivars for Home Plantings" extension "G6005"`
- `"Growing Peaches in Maine" "Bulletin 2068" cultivar`
- `"Home Orchards" "Stone Fruit Variety Selection" "Penn State Extension"`
- `"Growing Apricots, Cherries, Peaches" "A3639" "Wisconsin"`

When the publication number is unknown, try:

- `"fruit cultivars" "[state]" extension`
- `"fruit tree cultivars" "[state]" extension`
- `"home orchards" "[crop]" "variety selection" extension`
- `"recommended varieties" "[crop]" "[state]" extension`
- `site:[institution-domain] "[crop]" cultivar extension`

## Terms That Worked

These terms tended to find sources with cultivar-level data rather than general
home gardening pages:

- `fruit cultivars`
- `fruit tree cultivars`
- `small- and tree-fruit cultivars`
- `home orchards`
- `variety selection`
- `recommended varieties`
- `apple cultivars`
- `stone fruit variety selection`
- `publication`
- `bulletin`
- `extension`

Publication numbers are especially useful. Examples found in current candidate
work include `MF1028`, `G2354`, `PM453`, `EB-2023-0684`, `G6005`, `A3639`, and
`Bulletin 2068`.

## Useful Domain Filters

Prefer official extension and university domains when known:

- `site:bookstore.ksre.ksu.edu`
- `site:extensionpubs.unl.edu`
- `site:shop.iastate.edu`
- `site:extension.umd.edu`
- `site:extension.psu.edu`
- `site:extension.umaine.edu`
- `site:lsuagcenter.com`
- `site:learningstore.extension.wisc.edu`

If an official result is missing from general search, search inside the
institution site if it has its own publication search.

## Noisy Patterns

These searches can surface leads, but usually include many nursery, blog, SEO,
or summary pages:

- `best fruit trees for [state]`
- `fruit varieties home garden extension pdf`
- `state extension fruit cultivars home plantings apple peach grape`
- `site:edu fruit varieties home orchard cultivar PDF`

Use noisy searches to discover title words, publication numbers, or institution
names, then re-search with an official domain filter before adding anything to
the queue.

## Download Link Discovery

Some sources are description pages, store pages, or publication catalog entries
instead of direct PDFs. For those:

1. Open the landing page and look for `PDF`, `download`, `publication`, `view`,
   or `full text` links.
2. Record the landing page in `source_url`.
3. Record the direct file in `download_url` when one is found.
4. If a newer file is found for an already-known reference, flag it for hash
   comparison rather than silently replacing the old asset.
5. If only a description page is available, keep it as a candidate when it has
   enough title, institution, and cultivar evidence to revisit later.

## Checks

- The URL opens without requiring an account.
- The source names specific cultivars or varieties.
- The source has enough metadata to cite later.
- Search results were followed to a source page or file, not queued directly
  from search snippets.

## Open Questions

- Should low-priority sources be kept, discarded, or archived separately?
