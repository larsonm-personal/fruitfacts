#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
file=2021_Gaz_zcta_national.txt
[[ ! -s $file ]] || exit 0
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
curl --fail --location --retry 2 --connect-timeout 15 --max-time 120 \
  https://www2.census.gov/geo/docs/maps-data/data/gazetteer/2021_Gazetteer/2021_Gaz_zcta_national.zip \
  -o "$scratch/data.zip"
unzip -p "$scratch/data.zip" "$file" > "$scratch/$file"
[[ $(wc -l < "$scratch/$file") -gt 33000 ]] || { echo 'Incomplete gazetteer' >&2; exit 1; }
mv "$scratch/$file" "$file"
