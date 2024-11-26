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
--release` will run them.

### Architecture

The Rust `backend` crate gets compiled to WASM, with generally type-unsafe APIs
in `lib.rs`. The Svelte frontend calls these in the main thread (though moving
to web workers later is a possibility). The "important" state is kept in the
backend, while the frontend has more ephemeral UI state. The main state is
serialized as GeoJSON and kept in the user's browser local storage. The
frontend is effectively a single page app, with the `Mode` toggling between
different pages.
