# Low-traffic neighbourhood tool v2

LTNs are a traffic management measure to stop excessive through-traffic cutting
through residential streets. This is an interactive web app for rapidly
exploring their design.

- Global version: https://a-b-street.github.io/ltn (import anywhere from OpenStreetMap)
- England: https://a-b-street.github.io/ltn/england.html
- Scotland: https://cnt.scot
- Read the user guide: https://a-b-street.github.io/ltn/user_guide.html

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
Dustin on assets.od2net.org and assets.cnt.scot.

If you're developing locally, you can cache assets locally by running
`bin/download-local-dev-data.sh`.

### Translations

If you're translating the app, you'll be editing `.po` files in
`web/src/locales/`. You have to refresh the entire page to see changes. If
you're using a tool to edit the `.po` file that's handy, please share it so we
can list it here! And don't forget to add yourself to the credits in
`web/src/About.svelte`.

When the app changes and new strings to translate appear, they should be
detected and added to the `.po` automatically. Run `npx wuchale init` if
something goes wrong.

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
