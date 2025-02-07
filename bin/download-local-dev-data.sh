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

cd "${APP_ROOT}/web/public"

# Scotland specific data
jq '.features[] | .properties.kind + "_" + .properties.name' ../../data_prep/scotland/boundaries.geojson | sed 's/"//g' | while read x; do
    download_to_subdir cnt_boundaries "https://assets.od2net.org/cnt_boundaries/$x.geojson"
    download_to_subdir cnt_osm "https://assets.od2net.org/cnt_osm/$x.osm.pbf"
    # OD Demand model
    download_to_subdir cnt_demand "https://assets.od2net.org/cnt_demand/demand_$x.bin"
done

for x in cbd.pmtiles gp_practices.geojson hospitals.geojson population.pmtiles route_network.pmtiles schools.geojson; do
    download_to_subdir cnt_layers https://assets.od2net.org/cnt_layers/$x
done
