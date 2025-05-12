# "normally" the app downloads the latest data from Overpass API based on the
# user's clipped areas.
#
# For expediency and deterministic tests, this script downloads some
# pre-configured areas, which the web app will load from localhost.

APP_ROOT=$(git rev-parse --show-toplevel)

./bin/download-local-test-data.sh

download_to_subdir() {
    local subdir=$1
    local url=$2

    mkdir -p "$subdir"
    (wget -P "$subdir" --timestamping "$url" && echo "✅ (CNT) $url") \
        || echo "❌ (CNT) Download failed: $url"
}

mkdir -p "${APP_ROOT}/web/public"
cd "${APP_ROOT}/web/public"

# Scotland specific data
jq '.features[] | .properties.kind + "_" + .properties.name' ../../data_prep/scotland/boundaries.geojson | sed 's/"//g' | while read x; do
    download_to_subdir cnt/boundaries "https://assets.cnt.scot/prod/boundaries/$x.geojson"
    download_to_subdir cnt/osm "https://assets.cnt.scot/prod/osm/$x.osm.pbf"
    download_to_subdir cnt/demand "https://assets.cnt.scot/prod/demand/$x.bin"
    download_to_subdir cnt/prioritization "https://assets.cnt.scot/prod/prioritization/$x.bin"
done

for x in bus_routes.pmtiles cbd.pmtiles population.pmtiles railways.geojson route_network.pmtiles stats19.pmtiles; do
    download_to_subdir cnt/layers https://assets.cnt.scot/prod/layers/$x
done
