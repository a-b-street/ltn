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

  # Manually register and download GeoJSON from https://data.spatialhub.scot/dataset/gp_practices-i://data.spatialhub.scot/dataset/nhs_hospitals-is
  ogr2ogr tmp/hospitals.geojson \
          -t_srs EPSG:4326 \
          $2 \
          -sql 'SELECT sitename AS name FROM "NHS_Hospitals_-_Scotland"'

  # The bboxes or something else included are breaking parsing, so clean these up
  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: { name: .properties.name } }] }' tmp/gp_practices.geojson > tmp.gj
  mv -f tmp.gj $OUT/gp_practices.geojson

  jq '{ type: "FeatureCollection", features: [.features[] | { type: "Feature", geometry: .geometry, properties: { name: .properties.name } }] }' tmp/hospitals.geojson > tmp.gj
  mv -f tmp.gj $OUT/hospitals.geojson
}

function cbd {
  wget https://nptscot.blob.core.windows.net/pmtiles/cbd_layer_2024-12-01.pmtiles -O $OUT/cbd.pmtiles
}

function population {
  # From https://www.data.gov.uk/dataset/1102bf85-ed49-440a-b211-da87e8d752eb/scottish-index-of-multiple-deprivation-simd-2020
  wget https://maps.gov.scot/ATOM/shapefiles/SG_SIMD_2020.zip
  unzip SG_SIMD_2020.zip
  ogr2ogr tmp/population.geojson \
          -t_srs EPSG:4326 \
          SG_SIMD_2020.shp \
          -nlt PROMOTE_TO_MULTI \
          -sql 'SELECT DataZone, Rankv2 as rank, Percentv2 as percentile, SAPE2017 as population, OGR_GEOM_AREA as area FROM SG_SIMD_2020'
  rm -f SG_SIMD_2020* SIMD2020v2*xlsx
  tippecanoe tmp/population.geojson --generate-ids -l population -o $out/population.pmtiles
}

# Uncomment each as needed
#route_network
#schools
#gp_and_hospitals ~/Downloads/GP_Practices_-_Scotland.json ~/Downloads/NHS_Hospitals_-_Scotland.json
#cbd
#population
