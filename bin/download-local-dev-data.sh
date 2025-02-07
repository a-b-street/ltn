# "normally" the app downloads the latest data from Overpass API based on the
# user's clipped areas.
#
# For expediency and deterministic tests, this script downloads some
# pre-configured areas, which the web app will load from localhost.

cd "$(git rev-parse --show-toplevel)"

download_to_subdir() {
    local subdir=$1
    local url=$2

    mkdir -p "$subdir"
    (wget -P "$subdir" --timestamping "$url" && echo "✅ $url") \
        || echo "❌ Download failed: $url"
}

cd web/public

download_to_subdir osm https://assets.od2net.org/severance_pbfs/areas.json

# Global data used for tests and demo data
AREAS="bristol edinburgh strasbourg ut_dallas"
for x in $AREAS; do
    download_to_subdir severance_pbfs "https://assets.od2net.org/severance_pbfs/$x.pbf"
    download_to_subdir boundaries "https://assets.od2net.org/boundaries/$x.geojson"
done

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
