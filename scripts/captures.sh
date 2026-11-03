#!/usr/bin/env bash
# Saves one screen's 8 captures in captures/: 4 formats, light and dark.
#
#   scripts/captures.sh NAME [rubrique|onglet] [INDEX]
#   scripts/captures.sh 00-evenement onglet 1
#
# macOS limits a window to the screen height. With this scale factor, the tallest
# format (1180 px) still fits on the screen.
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="$PATH:$HOME/.cargo/bin"
export SLINT_SCALE_FACTOR=1.5

nom="$1"
shift

cargo build --quiet
mkdir -p captures

#        name           width height sizes
formats=("telephone     402   874    tactile"
         "ipad-portrait 820   1180   tactile"
         "ipad-paysage  1180  820    tactile"
         "ordinateur    1440  900    souris")

for format in "${formats[@]}"; do
    read -r appareil largeur hauteur tailles <<<"$format"
    for theme in clair sombre; do
        target/debug/maquette --capture "captures/$nom-$appareil-$theme.png" \
            "$theme" "$largeur" "$hauteur" "$tailles" "$@"
        echo "captures/$nom-$appareil-$theme.png"
    done
done
