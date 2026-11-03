#!/bin/sh
# Run the desktop app with the craft-fonts bundle embedded (see docs/development.md, "Fonts").
# Clones storytold/craft-fonts next to this repo on first use; later runs reuse it.
# Arguments go to the app, e.g. `scripts/run-with-fonts.sh --new-instance image.psd`.
set -eu
cd "$(dirname "$0")/.."
fonts="$(cd .. && pwd)/craft-fonts"
if [ ! -f "$fonts/fonts/manifest.txt" ]; then
    git clone --depth 1 https://github.com/storytold/craft-fonts.git "$fonts"
fi
CRAFT_FONTS_DIR="$fonts" exec cargo run --release -p photocraft -- "$@"
