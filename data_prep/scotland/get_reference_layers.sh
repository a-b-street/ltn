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
  # Strip most tag
  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: { name: .properties.name } }] }' tmp/railways.geojson > $OUT/railways.geojson
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

echo "For maintainer only:"
echo "  mv $OUT/* ~/cloudflare_sync/cnt_layers/"
echo "And then upload"
