#!/usr/bin/env bash
set -e
cd "$(dirname "${BASH_SOURCE[0]}")"

wget "https://www2.census.gov/geo/docs/maps-data/data/gazetteer/2021_Gazetteer/2021_Gaz_zcta_national.zip" -O temp.zip
unzip temp.zip
rm -f temp.zip
