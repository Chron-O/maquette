#!/usr/bin/env bash
# Builds what GitHub Pages publishes, in site/: the web version at the root,
# the frozen HTML prototype in html/. Also used by .github/workflows/pages.yml.
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="$PATH:$HOME/.cargo/bin"

wasm-pack build --release --target web --no-typescript --out-dir pkg

rm -rf site
mkdir -p site/pkg
cp web/index.html web/favicon.png site/
cp pkg/maquette.js pkg/maquette_bg.wasm site/pkg/
cp -R html site/html
du -sh site/pkg/maquette_bg.wasm
