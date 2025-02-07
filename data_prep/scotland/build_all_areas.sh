#!/bin/bash
#
# This generates an osm.pbf clip per LAD and TA boundary. You'll need some dependencies:
#
# - wget, python3
# - osmium (https://osmcode.org/osmium-tool)

set -e

function split_osm {
        mkdir -p tmp
        cd tmp

        # Download Scotland OSM data
        if [ ! -f scotland-latest.osm.pbf ]; then
          wget https://download.geofabrik.de/europe/united-kingdom/scotland-latest.osm.pbf
        fi

        # Generate config for osmium
        mkdir -p osm_out
        mkdir -p osmium_inputs
        cd osmium_inputs
        python3 ../../geojson_to_osmium_extracts.py ../../boundaries.geojson --output_dir=../osm_out/ --batch_size=10

        # Create an osm.pbf file per boundary
        for batch in osmium_cfg_*; do
          time osmium extract -v -c $batch ../scotland-latest.osm.pbf
        done

        cd ../..
}

split_osm

echo "For maintainer only:"
echo "  mv tmp/osm_out/* ~/cloudflare_sync/cnt_osm/"
echo "  cp tmp/osmium_inputs/*geojson ~/cloudflare_sync/cnt_boundaries/"
echo "And then upload"
