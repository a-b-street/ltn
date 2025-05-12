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

        # Gzip everything
        cd ../osm_out
        for x in *; do
          gzip "$x"
        done
}

split_osm

echo "To use these files:"
echo "  mkdir -p ../../web/public/cnt/osm ../../web/public/cnt/boundaries"
echo "  mv tmp/osm_out/* ../../web/public/cnt/osm/"
echo "  mv tmp/osmium_inputs/*geojson ../../web/public/cnt/boundaries/"
