# Low-traffic neighbourhood tool v2

This is a new version of [ltn.abstreet.org](https://ltn.abstreet.org). It
doesn't have all features from v1 yet, but it solves many problems with the
original. Please try both versions and give feedback.

## Developer docs

### Installation

You'll need:
[npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm),
[wasm-pack](https://github.com/rustwasm/wasm-pack), and
[cargo](https://www.rust-lang.org/tools/install).

`cd web`, and then:

- `npm ci` to install dependencies (`ci` to make sure the versions in
  `package-lock.json` are used)
- `npm run wasm` to rebuild the Rust backend
  - vite doesn't automatically rebuild when you edit things
- `npm run dev` to run locally
  - Changes to the Svelte/CSS usually auto-reload in your browser
- `npm run fmt` to auto-format code
- `npm run check` to see TypeScript errors

### Faster local development

The tool operates on fixed study areas, generated from clips of OSM data.
Custom areas imported by the user are always retrieved from the Overpass API,
with the latest OSM data. For speed and for deterministic tests, there are also
"built-in" study areas, consisting of pre-clipped osm.pbf files that do not
automatically use the latest OSM data. These are manually managed and hosted by
Dustin on assets.od2net.org.

If you're developing locally, you can avoid hitting od2net.org by setting up
three directories in `web/public/`: `osm`, `boundaries`, and `cnt_demand`. If
`web/public/osm/areas.json` exists, then the Svelte app will load from
localhost, not od2net.org. You can copy from od2net.org to set this up,
choosing what study areas to cache:

```
AREAS="bristol edinburgh strasbourg ut_dallas"

cd web/public
mkdir boundaries osm cnt_demand

cd osm
wget https://assets.od2net.org/severance_pbfs/areas.json
for x in $AREAS; do
  wget https://assets.od2net.org/severance_pbfs/$x.pbf
done

cd ../boundaries
for x in $AREAS; do
  wget https://assets.od2net.org/boundaries/$x.geojson
done
cd ..
```

There are three more directories particular to Scotland. To cache all of that data:

```
# Still in web/public

mkdir cnt_osm cnt_boundaries cnt_demand
jq '.features[] | .properties.kind + "_" + .properties.name' ../../data_prep/scotland/boundaries.geojson | sed 's/"//g' | while read x; do
  wget https://assets.od2net.org/cnt_boundaries/$x.geojson
  wget https://assets.od2net.org/cnt_osm/$x.osm.pbf
  wget https://assets.od2net.org/cnt_demand/$x.bin
  mv $x.geojson cnt_boundaries
  mv $x.osm.pbf cnt_osm
  mv $x.bin cnt_demand
done
```

### Tests

(Don't spend too much time looking after these particular tests; they've been
helpful to spot unexpected changes in calculating existing modal filters, but
they're often just noisy when cell geometry slightly changes.)

The `tests` directory has some diff-based tests. `cd backend; cargo test
--release` will run them. You'll need to follow the instructions above to
ensure you have `bristol` and `strasbourg` areas set up. To accept the diffs,
just commit the changed files.

There's a few ways to understand / verify the diffs.

First, you can manually try the tool
[before](https://a-b-street.github.io/ltn/) and after (running locally). You
can load a `.geojson` file from `tests/` as a project, then click the
neighbourhood boundary (Bristol has two tests, Strasbourg just one). Then just
visually compare things -- cells, shortcuts, and existing filters.

You can also try just diffing the output GeoJSON file as text. They're stored
without pretty-printed newlines/indentation to save size in git, but you can do
something like this to view, assuming you have `jq` and `meld` (or another diff
tool):

```
function json_diff {
  cat $1 | jq > /tmp/after.json
  git show HEAD^:./$1 | jq > /tmp/before.json
  meld /tmp/before.json /tmp/after.json
}

cd tests/output
json_diff bristol_west.geojson
```

Finally, if you have the before and after output GeoJSON file (`json_diff`
creates `/tmp/before.geojson` and `/tmp/after.geojson`), then you can try
exploring in [GeoDiffr](https://dabreegster.github.io/geodiffr). You'll want to
"remove unchanged features", then try to understand the remaining changes. This
tool is not very sophisticated; it's only helpful sometimes when filters
change, but it's less useful if big cell polygons change.

### Architecture

The Rust `backend` crate gets compiled to WASM, with generally type-unsafe APIs
in `lib.rs`. On the frontend side, `wasm.ts` wraps these APIs in nicer TS APIs.
The Svelte frontend calls APIs in the main thread (though moving to web workers
later is a possibility). The "important" state is kept in the backend, while
the frontend has more ephemeral UI state. The main state is serialized as
GeoJSON and kept in the user's browser local storage. The frontend is
effectively a single page app, with the `Mode` toggling between different
pages.
