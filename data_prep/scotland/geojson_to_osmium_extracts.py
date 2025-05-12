import argparse
import json


# This tool takes a GeoJSON with many features, and prepares Osmium to extract
# a boundary for each one. It's faster to create multiple PBF extracts (based
# on --batch_size) with one osmium pass, so this script writes a config that's
# then later run. The output filenames are based on a feature's `kind` and
# `name` properties. See
# https://osmcode.org/osmium-tool/manual.html#creating-geographic-extracts
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "input", help="Path to a GeoJSON file with a FeatureCollection", type=str
    )
    parser.add_argument(
        "--config_output",
        default="osmium_cfg_%d.json",
        help="Name of the osmium JSON config files to create, with '%d' as a parameter",
        type=str,
    )
    parser.add_argument(
        "--batch_size",
        help="How many areas to extract in each osmium run. Too many will eat your RAM.",
        type=int,
    )
    parser.add_argument(
        "--output_dir",
        help="Where to write the .osm output files",
        type=str,
    )
    args = parser.parse_args()

    config = {"directory": args.output_dir, "extracts": []}
    with open(args.input) as f:
        gj = json.load(f)
        num_batches = 0

        for feature in gj["features"]:
            name = feature["properties"]["kind"] + "_" + feature["properties"]["name"]

            with open(f"{name}.geojson", "w") as f:
                f.write(json.dumps(feature))
            config["extracts"].append(
                {
                    "output": f"{name}.osm.pbf",
                    "output_format": "pbf,add_metadata=false",
                    "polygon": {"file_name": f"{name}.geojson", "file_type": "geojson"},
                }
            )
            if len(config["extracts"]) == args.batch_size:
                with open(args.config_output % num_batches, "w") as f:
                    f.write(json.dumps(config))
                config["extracts"] = []
                num_batches += 1

        with open(args.config_output % num_batches, "w") as f:
            f.write(json.dumps(config))


if __name__ == "__main__":
    main()
