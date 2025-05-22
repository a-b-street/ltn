## Study area boundaries

`boundaries.geojson` comes from <https://geoportal.statistics.gov.uk/datasets/ons::local-authority-districts-may-2024-boundaries-uk-buc-2/about>. After downloading the GeoJSON file (unavailable at a permalink):

```
# Filter for England LADs, then clean up properties and precision
mapshaper ~/Downloads/Local_Authority_Districts_May_2024_Boundaries_UK_BUC_6795818826918236547.geojson -filter 'LAD24CD.startsWith("E")' -each 'name=LAD24NM, delete FID, delete LAD24CD, delete LAD24NM, delete LAD24NMW, delete BNG_E, delete BNG_N, delete LONG, delete LAT, delete GlobalID, kind="LAD"' -o precision=0.000001 tmp.geojson

# Coerce everything to a MultiPolygon
ogr2ogr boundaries.geojson -nlt PROMOTE_TO_MULTI tmp.geojson

rm -f tmp.geojson
```

## OD data

```
./get_input.sh

mkdir -p ../../web/public/england/demand

# If needed, `cd ../common; cargo build --release; cd ../england`
../../target/release/generate_od \
  --study-area-boundaries boundaries.geojson \
  --od-zones zones.geojson \
  --od-csv od.csv \
  --out-dir ../../web/public/england/demand/
```
