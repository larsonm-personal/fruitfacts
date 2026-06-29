# Association Patterns

This is the shared helper for recurring ways FruitFacts maps source material
into JSON5 reference files. Update it when comparing sources to existing
encodings or creating new encodings.

Reviewed examples so far:

First pass:

- `plant_database/references/Arkansas/Small Fruit Cultivar Recommendations for Arkansas.json5`
- `plant_database/references/Arkansas/Arkansas Table Grape Cultivars.json5`
- `plant_database/references/commercial/ACN Maturity Chart.json5`
- `plant_database/references/commercial/DWN/DWN Retail Harvest Times.json5`

Second pass:

- `plant_database/references/Florida/MG36800 Low-Chill Apple Cultivars for North Florida and North Central Florida.json5`
- `plant_database/references/Florida/HS895 Growing Plums in Florida.json5`
- `plant_database/references/New Jersey/FS1201 Yellow-Fleshed Peach Varieties for New Jersey Commercial and Home Orchardists.json5`
- `plant_database/references/Oregon/OSU EC 1181 Selecting Peach and Nectarine Varieties for the Willamette Valley.json5`
- `plant_database/references/Wisconsin/A2105 Apple Cultivars for Wisconsin.json5`

## Source Checks

- Arkansas small fruit recommendations were checked against the University of
  Arkansas berries page and its linked cultivar recommendation guide:
  `https://www.uaex.uada.edu/yard-garden/fruits-nuts/berries.aspx`
- Arkansas table grape entries were checked against the Extension table grape
  article and the University of Arkansas fruit breeding grape page:
  `https://grapes.extension.org/arkansas-table-grape-cultivars/`
  `https://aaes.uada.edu/fruit-breeding/grapes/`
- ACN maturity entries were checked against the source PDF:
  `https://acnursery.com/wp-content/uploads/2022/01/ACN-maturity-chart.pdf`
- Dave Wilson harvest entries were checked against the current harvest chart
  landing page and linked chart image. The PDF URL currently encoded in the
  reference returned 404 during this review:
  `https://www.davewilson.com/nurseries/growing-guides/`
- Florida low-chill apple entries were checked against the current AskIFAS page.
  The encoded `edis.ifas.ufl.edu` URL still resolves:
  `https://edis.ifas.ufl.edu/publication/MG368`
- Florida plum entries were checked against the current AskIFAS page:
  `https://edis.ifas.ufl.edu/publication/hs250`
- Rutgers yellow-fleshed peach entries were checked against the NJAES page:
  `https://njaes.rutgers.edu/fs1201/`
  The current page supports the harvest and bacterial spot fields, but did not
  make the encoded Cream Ridge location obvious during this review.
- Oregon peach and nectarine entries were checked against the current OSU
  Extension page and PDF. The encoded old catalog URL returned 404 during this
  review:
  `https://extension.oregonstate.edu/catalog/ec-1181-selecting-peach-nectarine-varieties-willamette-valley`
  `https://extension.oregonstate.edu/sites/extd8/files/catalog/auto/EC1181.pdf`
- Wisconsin apple entries were checked against the source PDF:
  `https://barron.extension.wisc.edu/files/2021/09/Apple-Cultivars-for-WI-A2105.pdf`

## Source Shapes

- Narrative cultivar notes can map one paragraph or compact table row to one
  plant entry. Keep source wording in `description` when it carries adaptation,
  disease, fruit quality, pollination, or caution information.
- Recommendation lists with only cultivar names can use `names` when the source
  makes the same claim for every item in the group.
- Chart sources usually provide only cultivar, crop type, and harvest window.
  Use one plant entry per chart bar and avoid adding description text unless it
  comes from another cited source.
- Web dictionary pages can point individual entries back to source URLs in
  `description` when full detail has not yet been extracted.
- Narrative cultivar pages can be encoded by source heading when the heading
  carries the cultivar name and a harvest note, such as a parenthesized season.
- Tables that mix recommended, conditionally recommended, and not recommended
  cultivars need an explicit inclusion rule in a comment or `needs_help`.
- Appendix charts can contain entries that are absent from the main narrative.
  Mark the collection as incomplete when only the narrative or only the chart
  has been encoded.
- HTML tables can contain extraction artifacts that still reflect source
  structure, such as wrapped cultivar names appearing as fragment rows or
  section headings appearing as single-cell rows. Repair these mechanically
  when the intended source row is clear, and keep source-specific repairs
  visible in descriptions or comments when the source table itself is
  ambiguous.

## Locations

- If the source gives region-specific recommendations, define one location for
  each named region and use `short_name` when source tables use compact labels.
- If all rows share one trial station, nursery site, or representative source
  location, define one location at the collection level and omit per-plant
  `locations`.
- If a source gives statewide adaptation but no exact site, use a broad
  representative location and explain the choice in an inline note or
  `needs_help`.
- If a source gives harvest timing for a specific site, prefer that site over
  a generic state location. Examples include Hickman, CA for Dave Wilson and
  Aspers, PA for ACN.
- If a plant row is only recommended for some of the collection locations, use
  `locations: ["short name"]` or location-name harvest maps rather than
  duplicating the plant.
- If source tables use location codes, define the full locations once at the
  collection level and use the codes in each plant row.
- If a source gives a region-level harvest caveat, such as one part of a valley
  ripening one to two weeks earlier, do not flatten the caveat into every row.
  Keep the caveat in source notes, collection notes, or follow-up work.
- If the current source page does not name an exact trial site, avoid treating a
  nearby research farm as more precise than the source supports.

## Categories

- Use top-level `categories` when a source repeats meaningful group headings and
  those headings have reusable explanations.
- Use per-plant `category` when the source row belongs to a named group such as
  low chill, disease resistant, table grape, wine grape, juice grape, white
  peach, or flat peach.
- Use `category_description` when the source explains why the category matters.
  Do not invent category meaning from a heading alone.
- For chart sections that mainly group by crop or flesh color, prefer comments
  during rough encoding. Promote them to `category` only if the category should
  be queryable later.
- Use `tags` for source section labels such as yellow peach, yellow nectarine,
  or disease resistant when the label is useful but not central enough to be a
  normalized category.
- Home orchard suggestions are a separate recommendation dimension from
  commercial grower suitability. Preserve that distinction in `description`,
  `category`, `tags`, or a future dedicated field.

## Plant Names

- Use `name` when the source row is one cultivar or when one name should be the
  canonical searchable cultivar name.
- Use `names` when the source presents a flat list of equivalent entries that
  share the same type, location, category, and notes.
- Put trademark, marketing, selection, and patent names in `AKA` when a more
  stable cultivar or patent name is known.
- If the source uses only a marketing name and no cultivar name is known, keep
  the visible source name as `name` and add a note in `description` or a comment.
- Use comments for uncertain synonym choices when a source, patent, nursery
  page, or chart appears inconsistent.
- Preserve source capitalization only when it appears meaningful. Otherwise,
  follow existing FruitFacts cultivar naming style.
- If the source uses a trademark or brand name but a stable cultivar name is
  known from another authoritative source, keep the cultivar name in `name` and
  the source-facing name in `AKA`.
- If a source typo is corrected during encoding, leave a short comment with the
  source spelling or the reason for the correction.

## Harvest Times

- Preserve source precision. Exact dates can be encoded as exact dates, date
  ranges as ranges, and vague phrases as phrases.
- Do not convert early, mid, late, or season-only source wording into exact
  dates unless the source itself provides the conversion.
- When the source gives a location-specific harvest phrase, encode the harvest
  time and note the location context if it is not already captured by the
  collection location.
- For matrix sources with different harvest times by region, use the existing
  per-location object style:

```json5
locations: [
    { "San Joaquin Valley": "late Jun" },
    { "North Coast": "late Jul" }
]
```

- For bar-chart sources, manually transcribed date ranges are acceptable when
  the chart axis is clear. Keep `harvest_time_devalue_factor` if the chart is
  useful but less precise than source text or trial data.
- Use `NA` or omit a location value only when the source explicitly says the
  variety is not adapted, not recommended, or not applicable there.
- Use `harvest_time_relative` when the source gives timing as an offset from an
  anchor cultivar, such as `Redhaven -14 days`.
- Use `harvest_time_unparsed` when the source gives a relative phrase that has
  multiple anchors, incomplete context, or wording the importer should not try
  to normalize yet.
- If a source gives week-of-month timing, prefer preserving the source phrase.
  If converting to representative dates for charting, put the conversion rule
  in a nearby comment.
- Do not turn vague phrases such as a few days before into numeric offsets
  unless the file records that the number is an encoder estimate.

## Recommendation And Trial Meaning

- A recommendation guide generally means the plant is suitable for the stated
  region unless a description says otherwise.
- A trial publication can include neutral, poor, or experimental entries. Do not
  treat every trialed cultivar as recommended unless the source does.
- Phrases such as for trial, not recommended outside a region, susceptible, or
  uneven ripening belong in `description`, not just in private notes.
- Use `type` values from `plant_database/types.json5`, even when the source
  uses broader crop headings.
- Use `needs_help: true` when only one crop section or subset of a source has
  been encoded.
- Not recommended entries can still be useful when the source explains why.
  Include them with `not_recommended_reason` when the warning is useful for
  gardeners or for search results.
- If not recommended entries are excluded, leave a collection comment or
  `needs_help` note so future work does not mistake omission for source absence.

## Disease, Patent, And Trait Fields

- Use structured fields such as `disease_resistance`, `seedless`, `color`,
  `patent`, and `released` when the source or a linked authoritative source
  provides clear values.
- Keep disease wording source-like. For example, resistant, moderately
  susceptible, and occasional occurrence should not be flattened into one
  invented scale without a separate normalization pass.
- Patent and trademark research may come from outside the main horticulture
  source. When that happens, keep the supporting patent URL in `patent` and
  note uncertainty in a comment or `description`.
- Source abbreviations such as BLS, CAR, FB, S, VS, R, MR, and VR can be stored
  directly when the source uses them consistently. Keep a legend in the source
  file or a guide note before doing broad normalization.
- Traits such as chill hours, fruit development period, storage life, hardiness,
  self-fertility, and strains should remain structured when the source states
  them clearly enough.

## Citation And Provenance Notes

- Keep the collection `url` pointed at the most useful source page or
  downloadable source file available at encoding time.
- If a source page links to a PDF, prefer the PDF for stable content and keep
  the landing page in notes when it helps future recovery.
- If the original URL is dead, blocked, or redirects, note that in the guide or
  candidate queue rather than silently replacing provenance.
- Keep `accessed` dates for sources likely to move, especially nursery pages and
  web dictionaries.
- DVC pointer files are not enough for review. If the actual PDF is unavailable,
  prefer a reachable source URL, an archived copy, or mark the reference for
  provenance follow-up.
- Extension systems move old URLs. If the encoded URL now redirects or returns
  404 but the same publication is available at a new official URL, record that
  in source-check notes before changing the data file.
- If a live source has a newer reviewed date than the encoded `published` date,
  treat that as a metadata follow-up unless the content has been rechecked
  against the newer edition.

## Follow-Up Patterns To Learn

- Multi-location extension matrices with recommendation marks but no harvest
  dates.
- Trial reports where locations, years, and cultivar performance are all
  separate dimensions.
- Sources with both rootstock and cultivar rows.
- OCR-heavy PDFs where table structure is visible but text extraction is weak.
