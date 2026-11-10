#!/usr/bin/env bash
# Saves one screen's 8 captures in captures/: 4 formats, light and dark.
#
#   scripts/captures.sh NAME [club|tab|live] [INDEX]
#   scripts/captures.sh 00-event tab 1
#   CAPTURE_TEST_DATA=1 scripts/captures.sh 01-home-test-data
#
# macOS limits a window to the screen height. With this scale factor, the tallest
# format (1180 px) still fits on the screen.
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="$PATH:$HOME/.cargo/bin"
export SLINT_SCALE_FACTOR=1.5

name="$1"
shift

cargo build --quiet
mkdir -p captures

#        name           width height sizes
formats=("phone          402   874    touch"
         "ipad-portrait  820   1180   touch"
         "ipad-landscape 1180  820    touch"
         "desktop        1440  900    mouse")

for format in "${formats[@]}"; do
    read -r device width height sizes <<<"$format"
    for theme in light dark; do
        target/debug/maquette --capture "captures/$name-$device-$theme.png" \
            "$theme" "$width" "$height" "$sizes" "$@"
        echo "captures/$name-$device-$theme.png"
    done
done
