#!/bin/bash
# This script is run by Cloudflare Pages for deployment. It closely mimics .github/workflows/web.yml.

set -e
set -x

# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build the app
cd web
npm ci
npm run wasm-release
npm run build --if-present

# TODO Vite suddenly broke and I have no idea why. Hack around it.
# new URL('backend_bg.wasm', import.meta.url) doesn't exist at build time, it will remain unchanged to be resolved at runtime
ln -s assets/backend_bg.wasm dist/

# Overwrite the global version of the app with the CNT deployment
mv -f dist/cnt.html dist/index.html
rm -f dist/england.html
