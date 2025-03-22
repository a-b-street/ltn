#!/bin/bash
set -ex
download_to_subdir() {
  local subdir=$1
  local url=$2
  mkdir -p "$subdir"
  (wget -P "$subdir" --timestamping "$url" && echo "✅ $url") \
    || echo "❌ Download failed: $url"
}
if [ ! -d "input/scotland_settlements_shapefiles" ]; then
    download_to_subdir tmp https://www.nrscotland.gov.uk/media/2hsoadnx/shapefiles.zip
    unzip tmp/shapefiles.zip -d input/scotland_settlements_shapefiles
fi

# ogrinfo -so input/scotland_settlements_shapefiles/Settlements_2020_MHW.shp Settlements_2020_MHW

# Create GPKG and load settlements
# Assuming settlements is in EPSG:27700 (British National Grid)
ogr2ogr -f GPKG tmp/lad_settlements.gpkg \
    input/scotland_settlements_shapefiles/Settlements_2020_MHW.shp \
    -nlt PROMOTE_TO_MULTI \
    -nln settlements

# Add LAD boundaries to GPKG but reproject to match settlements projection
ogr2ogr -f GPKG -update -append tmp/lad_settlements.gpkg \
    boundaries.geojson \
    -t_srs EPSG:27700 \
    -nln lads

# Create output directory if it doesn't exist
mkdir -p output

ogr2ogr -f GeoJSON tmp/lad_settlements.geojson tmp/lad_settlements.gpkg \
  -sql "SELECT lads.name, lads.kind, ST_Union(ST_Intersection(lads.geom, settlements.geom)) AS geom
        FROM lads
        JOIN settlements
        ON ST_Intersects(lads.geom, settlements.geom)
        GROUP BY lads.name
        " \
  -t_srs EPSG:4326 \
  -nlt PROMOTE_TO_MULTI \
  -nln lad_settlements

