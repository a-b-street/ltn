This code is specific to Scotland, adapted from the Network Planning Tool project.

`boundaries.geojson` comes from <https://github.com/nptscot/npw/tree/main/data_prep>, turning polygons into multipolygons by doing `ogr2ogr boundaries.geojson -nlt PROMOTE_TO_MULTI path/to/npw/boundaries.geojson`

```
# Manually download https://github.com/nptscot/inputdata/releases/download/v1/desire_lines_scotland.csv from internal GH repo
xsv select geo_code1,geo_code2,car_driver,car_passenger ~/Downloads/desire_lines_scotland.csv > od.csv

# From https://spatialdata.gov.scot/geonetwork/srv/api/records/389787c0-697d-4824-9ca9-9ce8cb79d6f5
wget https://maps.gov.scot/ATOM/shapefiles/SG_IntermediateZoneBdry_2011.zip
unzip SG_IntermediateZoneBdry_2011.zip
ogr2ogr zones.geojson -t_srs EPSG:4326 -nlt PROMOTE_TO_MULTI SG_IntermediateZone_Bdry_2011.shp -sql 'SELECT InterZone as name FROM SG_IntermediateZone_Bdry_2011'
rm -f SG_IntermediateZone_Bdry_2011* SG_IntermediateZoneBdry_2011.zip

mkdir -p demand
cargo run --release --bin generate_od
```
