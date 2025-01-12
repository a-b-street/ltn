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

- `npm i` to install dependencies
- `npm run wasm` to rebuild the Rust backend
  - vite doesn't automatically rebuild when you edit things
- `npm run dev` to run locally
  - Changes to the Svelte/CSS usually auto-reload in your browser
- `npm run fmt` to auto-format code
- `npm run check` to see TypeScript errors

The `tests` directory has some diff-based tests. `cd backend; cargo test
--release` will run them. You'll need to follow the instructions below to
ensure you have `bristol` and `strasbourg` areas set up.

### Faster local development

The tool operates on fixed study areas, generated from clips of OSM data.
Custom areas imported by the user are always retrieved from the Overpass API,
with the latest OSM data. For speed and for deterministic tests, there are also
"built-in" study areas, consisting of pre-clipped osm.pbf files that do not
automatically use the latest OSM data. These are manually managed and hosted by
Dustin on assets.od2net.org.

If you're developing locally, you can avoid hitting od2net.org by setting up
two directories in `web/public/`: `osm` and `boundaries`. If
`web/public/osm/areas.json` exists, then the Svelte app will load from
localhost, not od2net.org. You can copy from od2net.org to set this up,
choosing what study areas to cache:

```
AREAS="bristol edinburgh strasbourg ut_dallas"

cd web/public
mkdir boundaries osm

cd osm
wget https://assets.od2net.org/severance_pbfs/areas.json
for x in $AREAS; do
  wget https://assets.od2net.org/severance_pbfs/$x.pbf
done

cd ../boundaries
for x in $AREAS; do
  wget https://assets.od2net.org/boundaries/$x.geojson
done
```

### Architecture

The Rust `backend` crate gets compiled to WASM, with generally type-unsafe APIs
in `lib.rs`. On the frontend side, `wasm.ts` wraps these APIs in nicer TS APIs.
The Svelte frontend calls APIs in the main thread (though moving to web workers
later is a possibility). The "important" state is kept in the backend, while
the frontend has more ephemeral UI state. The main state is serialized as
GeoJSON and kept in the user's browser local storage. The frontend is
effectively a single page app, with the `Mode` toggling between different
pages.
