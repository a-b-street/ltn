#!/bin/bash

set -e
set -x
OUT=out_layers
mkdir -p $OUT

function route_network {
  # From https://github.com/nptscot/outputdata/releases
  echo "Manually download https://github.com/nptscot/outputdata/releases/download/v2025-02-01/rnet_2025-02-01.pmtiles and save it to $OUT/route_network.pmtiles"
}

function schools {
  # From https://www.data.gov.uk/dataset/9a6f9d86-9698-4a5d-a2c8-89f3b212c52c/scottish-school-roll-and-locations
  wget https://maps.gov.scot/ATOM/shapefiles/SG_SchoolRoll_2023.zip
  unzip SG_SchoolRoll_2023.zip
  ogr2ogr tmp/schools.geojson \
          -t_srs EPSG:4326 \
          SG_SchoolRoll_2023/SG_SchoolRoll_2023.shp \
          -sql 'SELECT SchoolType AS type, SchoolName AS name, PupilRoll AS pupils FROM SG_SchoolRoll_2023'
  rm -rf SG_SchoolRoll_2023 SG_SchoolRoll_2023.zip
}

function gp_and_hospitals {
  # Manually register and download GeoJSON from https://data.spatialhub.scot/dataset/gp_practices-is
  ogr2ogr tmp/raw_gp_practices.geojson \
          -t_srs EPSG:4326 \
          $1 \
          -sql 'SELECT address AS name FROM "GP_Practices_-_Scotland"'

  # Manually register and download GeoJSON from https://data.spatialhub.scot/dataset/nhs_hospitals-is
  ogr2ogr tmp/raw_hospitals.geojson \
          -t_srs EPSG:4326 \
          $2 \
          -sql 'SELECT sitename AS name FROM "NHS_Hospitals_-_Scotland"'

  # The bboxes or something else included are breaking parsing, so clean these up
  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: { name: .properties.name } }] }' tmp/raw_gp_practices.geojson > tmp/gp_practices.geojson

  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: { name: .properties.name } }] }' tmp/raw_hospitals.geojson > tmp/hospitals.geojson
}

function cbd {
  wget https://nptscot.blob.core.windows.net/pmtiles/cbd_layer_2024-12-01.pmtiles -O $OUT/cbd.pmtiles
}

function railway_stations {
  get_scotland_osm
  osmium tags-filter tmp/scotland-latest.osm.pbf n/railway=station -o tmp/railways.osm.pbf
  osmium export tmp/railways.osm.pbf -o tmp/railways.geojson
  # Strip most tags
  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: { name: .properties.name } }] }' tmp/railways.geojson > $OUT/railways.geojson
}

function bus_routes {
  get_scotland_osm
  # Bus routes are represented as relations. Note many routes cross the same
  # way, but osmium only outputs the way once when we export to GeoJSON
  osmium tags-filter tmp/scotland-latest.osm.pbf r/route=bus -o tmp/bus_routes.osm.pbf
  # The relations also include stop positions as points. Only keep LineStrings,
  # representing roads.
  osmium export tmp/bus_routes.osm.pbf --geometry-type=linestring -o tmp/bus_routes_v1.geojson
  # Strip all tags
  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: {} }] }' tmp/bus_routes_v1.geojson > tmp/bus_routes_v2.geojson
  tippecanoe tmp/bus_routes_v2.geojson -zg -l bus_routes -o $OUT/bus_routes.pmtiles
}

function population {
  if [ ! -f tmp/SG_SIMD_2020.shp ]; then
      # From https://www.data.gov.uk/dataset/1102bf85-ed49-440a-b211-da87e8d752eb/scottish-index-of-multiple-deprivation-simd-2020
      download_to_subdir tmp https://maps.gov.scot/ATOM/shapefiles/SG_SIMD_2020.zip
      unzip -d tmp tmp/SG_SIMD_2020.zip
  else
    echo "✅ SG_SIMD_2020.shp already downloaded and extracted."
  fi

  _car_ownership
  # We need to join the _car_ownership CSV, so first import both datasets into sqlite

  ogr2ogr tmp/population.gpkg \
          tmp/SG_SIMD_2020.shp \
          -nln sg_simd_2020 \
          -nlt PROMOTE_TO_MULTI \
          -sql "SELECT
                    DataZone as id,
                    Rankv2 as imd_rank,
                    Percentv2 as imd_percentile,
                    SAPE2017 as population
                FROM SG_SIMD_2020"

  ogr2ogr -update tmp/population.gpkg \
          input/car_ownership_data_zones.csv \
          -nln car_ownership_data_zones \
          -sql "SELECT
                \"Number of cars or vans\" as data_zone_id,
                CAST(\"All occupied households\" AS INTEGER) as total_households,
                CAST(\"All occupied households\" AS INTEGER) - CAST(\"Number of cars or vans in household: No cars or vans\" AS INTEGER) as households_with_cars_or_vans
                FROM car_ownership_data_zones"

  ogr2ogr tmp/population.geojson \
          tmp/population.gpkg \
          -t_srs EPSG:4326 \
          -sql "SELECT
                    simd.geom,
                    simd.id,
                    simd.imd_rank,
                    simd.imd_percentile,
                    simd.population,
                    ST_Area(simd.geom) as area, -- Area calculation happens in original projection, before output to 4326
                    cars.total_households,
                    cars.households_with_cars_or_vans
                FROM sg_simd_2020 simd
                LEFT JOIN car_ownership_data_zones cars
                ON simd.id = cars.data_zone_id"

  tippecanoe tmp/population.geojson -zg --generate-ids -l population -o $OUT/population.pmtiles
}

function stats19 {
  # Use code from https://github.com/acteng/atip-data-prep/tree/main/stats19
  git submodule init
  git submodule update

  cd atip-data-prep/stats19

  # URLs from https://www.data.gov.uk/dataset/cb7ae6f0-4be6-4935-9277-47e5ce24a11f/road-safety-data
  download_to_subdir input https://data.dft.gov.uk/road-accidents-safety-data/dft-road-casualty-statistics-casualty-1979-latest-published-year.csv
  download_to_subdir input https://data.dft.gov.uk/road-accidents-safety-data/dft-road-casualty-statistics-collision-1979-latest-published-year.csv

  cargo run --release -- --only_pedestrians_and_cyclists

  download_to_subdir input https://raw.githubusercontent.com/georgique/world-geojson/refs/heads/develop/areas/united_kingdom/scotland.json
  mapshaper tmp/stats19.geojson -clip input/scotland.json -o tmp/stats19_clipped.geojson

  tippecanoe tmp/stats19_clipped.geojson \
      --force \
      --generate-ids \
      -l stats19 \
      -zg \
      --drop-densest-as-needed \
      --extend-zooms-if-still-dropping \
      -o ../../$OUT/stats19.pmtiles

  cd ../../
}

function _car_ownership() {
  if [ -f input/car_ownership_data_zones.csv ]; then
    echo "✅ Car ownership already downloaded."
    return 0
  fi

  # Nothing interesting happens in this method, we just print instructions that the user needs to manually follow.
  # We could omit it and put it in the readme, but maybe it's nice here for consistency with the other sources.
  cat <<EOS
🚗 Manually download car ownership CSV (buckle up, it's a wild ride!)
  1. go to https://www.scotlandscensus.gov.uk
  2. search for "UV405" that's the "Car or van availability" table
  3. choose "Data Zone 2011" area
  4. download the results as CSV (~48k records)
  5. mv \$downloaded_file ./input/car_ownership_data_zones.csv
  6. Manually delete the front-matter - it looks like this:
       1 SuperWEB2(tm)
       2 
       3 "household2022NotVisibleFromCatalogueScreen"
       4 "Scotland's Census 2022 - National Records of Scotland Table UV405 - Car or van availability All occupied households"
       5 "Intermediate Zone - Data Zone 2011 by Number of cars or vans"
       6 "Counting: Households"
       7 
       8 Filters:
       9 "Default Summation","Households"
      10 
  7. Manually delete the weird "second header" row which only has one column, after the first header row. It looks like this:
       2 "Intermediate Zone - Data Zone 2011",
  8. Manually delete the back-matter at the end of the csv - it looks like this:
    6989
    6990
    6991 "INFO","Data has been perturbed"
    6992
    6993
    6994 "Crown copyright 2024 For further information on variables, see https://www.scotlandscensus.gov.uk/about/metadata-a-to-z/ In order to protect against disclosure of personal information, Statistical Disclosure Control has been applied to the data/outputs.  Cells might not sum to sub totals and totals due to these Statistical Disclosure Controls. To find out more on this visit https://www.scotlandscensus.gov.uk/about/2022-census/statistical-methodology/statistical-disclosure-control/"
    6995
    6996 (c) Copyright WingArc Australia 2018
  9. Run a sanity check:
      $ wc -l input/car_ownership_data_zones.csv
      > 6977 input/car_ownership_data_zones.csv

      $ head -n 1 input/car_ownership_data_zones.csv
      > "Number of cars or vans","All occupied households","Number of cars or vans in household: No cars or vans","Number of cars or vans in household: One car or van","Number of cars or vans in household: Two cars or vans","Number of cars or vans in household: Three cars or vans","Number of cars or vans in household: Four or more cars or vans",

      $ tail -n1 input/car_ownership_data_zones.csv
      > "S01013481",393,43,163,148,28,8,
EOS
    exit 1
}

function get_scotland_osm {
  # Download Scotland OSM data
  if [ ! -f tmp/scotland-latest.osm.pbf ]; then
    download_to_subdir tmp https://download.geofabrik.de/europe/united-kingdom/scotland-latest.osm.pbf
  fi
}

download_to_subdir() {
  local subdir=$1
  local url=$2

  mkdir -p "$subdir"
  (wget -P "$subdir" --timestamping "$url" && echo "✅ $url") \
    || echo "❌ Download failed: $url"
}

# Uncomment each as needed
#route_network
#schools
#gp_and_hospitals ~/Downloads/GP_Practices_-_Scotland.json ~/Downloads/NHS_Hospitals_-_Scotland.json
#cbd
#railway_stations
#bus_routes
#population
#stats19

echo "Now, copy any layers you want for your local development:"
echo "  cp $OUT/* ../../web/public/cnt_layers/"
echo "For maintainer only:"
echo "  mv $OUT/* ~/cloudflare_sync/cnt_layers/"
echo "And then upload"
