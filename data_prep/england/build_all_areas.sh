#!/bin/bash
#
# This generates an osm.pbf clip per LAD. You'll need some dependencies:
#
# - wget, python3
# - osmium (https://osmcode.org/osmium-tool)

set -e

function split_osm {
        mkdir -p tmp
        cd tmp

        # Download England OSM data
        if [ ! -f england-latest.osm.pbf ]; then
          wget https://download.geofabrik.de/europe/united-kingdom/england-latest.osm.pbf
        fi

        # Generate config for osmium
        mkdir -p osm_out
        mkdir -p osmium_inputs
        cd osmium_inputs
        python3 ../../../geojson_to_osmium_extracts.py ../../boundaries.geojson --output_dir=../osm_out/ --batch_size=10

        # Create an osm.pbf file per boundary
        for batch in osmium_cfg_*; do
          time osmium extract -v -c $batch ../england-latest.osm.pbf
        done
}

split_osm
