This code is specific to Scotland, adapted from the Network Planning Tool project.

## Study area boundaries

`boundaries.geojson` comes from <https://github.com/nptscot/npw/tree/main/data_prep>, turning polygons into multipolygons by doing `ogr2ogr boundaries.geojson -nlt PROMOTE_TO_MULTI path/to/npw/boundaries.geojson`

## Prepare OD data

```
# Install https://github.com/medialab/xan if needed

# Manually download https://github.com/nptscot/inputdata/releases/download/v1/desire_lines_scotland.csv from internal GH repo
xan map 'car_driver + car_passenger' count ~/Downloads/desire_lines_scotland.csv | \
  xan rename zone1,zone2 -s geo_code1,geo_code2 | \
  xan select zone1,zone2,count > od.csv

# From https://spatialdata.gov.scot/geonetwork/srv/api/records/389787c0-697d-4824-9ca9-9ce8cb79d6f5
wget https://maps.gov.scot/ATOM/shapefiles/SG_IntermediateZoneBdry_2011.zip
unzip SG_IntermediateZoneBdry_2011.zip
ogr2ogr zones.geojson -t_srs EPSG:4326 -nlt PROMOTE_TO_MULTI SG_IntermediateZone_Bdry_2011.shp -sql 'SELECT InterZone as name FROM SG_IntermediateZone_Bdry_2011'
rm -f SG_IntermediateZone_Bdry_2011* SG_IntermediateZoneBdry_2011.zip
```

## Prepare other input data

```
./get_reference_layers.sh
./build_cnt_lad_settlements.sh
```

## Generating map model files

```
mkdir -p ../../web/public/cnt/maps_v1

cargo run --release -- \
  --study-area-boundaries boundaries.geojson \
  --osm-input-dir tmp/osm_out/ \
  --od-zones zones.geojson \
  --od-csv od.csv \
  --scotland-context-data \
  --out-dir ../../web/public/cnt/maps_v1/
```
