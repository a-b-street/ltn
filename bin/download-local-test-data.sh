# "normally" the app downloads the latest data from Overpass API based on the
# user's clipped areas.
#
# For expediency and deterministic tests, this script downloads some
# pre-configured areas, which the web app will load from localhost.

APP_ROOT=$(git rev-parse --show-toplevel)

download_to_subdir() {
    local subdir=$1
    local url=$2

    mkdir -p "$subdir"
    (wget -P "$subdir" --timestamping "$url" && echo "✅ $url") \
        || echo "❌ Download failed: $url"
}

cd "${APP_ROOT}/web/public"

download_to_subdir severance_pbfs https://assets.od2net.org/severance_pbfs/areas.json

# Global data used for tests and demo data
AREAS="bristol edinburgh strasbourg ut_dallas"
for x in $AREAS; do
    download_to_subdir severance_pbfs "https://assets.od2net.org/severance_pbfs/$x.pbf"
    download_to_subdir boundaries "https://assets.od2net.org/boundaries/$x.geojson"
done
