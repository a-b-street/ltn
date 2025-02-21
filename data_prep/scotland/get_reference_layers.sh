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
  ogr2ogr $OUT/schools.geojson \
          -t_srs EPSG:4326 \
          SG_SchoolRoll_2023/SG_SchoolRoll_2023.shp \
          -sql 'SELECT SchoolType AS type, SchoolName AS name, PupilRoll AS pupils FROM SG_SchoolRoll_2023'
  rm -rf SG_SchoolRoll_2023 SG_SchoolRoll_2023.zip
}

function gp_and_hospitals {
  # Manually register and download GeoJSON from https://data.spatialhub.scot/dataset/gp_practices-is
  ogr2ogr tmp/gp_practices.geojson \
          -t_srs EPSG:4326 \
          $1 \
          -sql 'SELECT address AS name FROM "GP_Practices_-_Scotland"'

  # Manually register and download GeoJSON from https://data.spatialhub.scot/dataset/nhs_hospitals-is
  ogr2ogr tmp/hospitals.geojson \
          -t_srs EPSG:4326 \
          $2 \
          -sql 'SELECT sitename AS name FROM "NHS_Hospitals_-_Scotland"'

  # The bboxes or something else included are breaking parsing, so clean these up
  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: { name: .properties.name } }] }' tmp/gp_practices.geojson > $OUT/gp_practices.geojson

  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: { name: .properties.name } }] }' tmp/hospitals.geojson > $OUT/hospitals.geojson
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
  # From https://www.data.gov.uk/dataset/1102bf85-ed49-440a-b211-da87e8d752eb/scottish-index-of-multiple-deprivation-simd-2020
  download_to_subdir tmp https://maps.gov.scot/ATOM/shapefiles/SG_SIMD_2020.zip
  cd tmp
  unzip SG_SIMD_2020.zip
  ogr2ogr population.geojson \
          -t_srs EPSG:4326 \
          SG_SIMD_2020.shp \
          -nlt PROMOTE_TO_MULTI \
          -sql 'SELECT DataZone as id, Rankv2 as imd_rank, Percentv2 as imd_percentile, SAPE2017 as population, OGR_GEOM_AREA as area FROM SG_SIMD_2020'
  rm -f SG_SIMD_2020* SIMD2020v2*xlsx
  cd ..
  tippecanoe tmp/population.geojson -zg --generate-ids -l population -o $OUT/population.pmtiles
}

function stats19 {
  # Use code from https://github.com/acteng/atip-data-prep/tree/main/stats19
  git submodule init
  git submodule update

  cd atip-data-prep/stats19
  mkdir -p input tmp

  # URLs from https://www.data.gov.uk/dataset/cb7ae6f0-4be6-4935-9277-47e5ce24a11f/road-safety-data
  if [ ! -e "input/dft-road-casualty-statistics-casualty-1979-latest-published-year.csv" ]; then
    download_to_subdir input https://data.dft.gov.uk/road-accidents-safety-data/dft-road-casualty-statistics-casualty-1979-latest-published-year.csv
  fi
  if [ ! -e "input/dft-road-casualty-statistics-collision-1979-latest-published-year.csv" ]; then
    download_to_subdir input https://data.dft.gov.uk/road-accidents-safety-data/dft-road-casualty-statistics-collision-1979-latest-published-year.csv
  fi

  cargo run --release

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
stats19

echo "For maintainer only:"
echo "  mv $OUT/* ~/cloudflare_sync/cnt_layers/"
echo "And then upload"
