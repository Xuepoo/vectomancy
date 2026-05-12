#!/bin/bash
set -e

echo "Building vectomancy..."
cargo build --release

OUT_DIR="../tmp/outputs"
mkdir -p "$OUT_DIR"

FORMATS=("png" "python" "desmos" "geogebra" "scratch" "kmplot" "latex" "wolfram" "json")
IMAGES=(
  "../tmp/assets/examples/01.jpg"
  "../tmp/assets/examples/02.jpg"
  "../tmp/assets/examples/03.jpg"
  "../tmp/assets/examples/04.jpg"
  "../tmp/assets/examples/05.jpg"
  "../tmp/assets/examples/06.jpg"
  "../tmp/assets/examples/07.jpg"
  "../tmp/assets/examples/fuji.png"
  "../tmp/assets/examples/hatsune_miku.jpg"
  "../tmp/assets/examples/伊坂幸太郎.jpg"
  "../tmp/assets/examples/十角馆事件.jpg"
  "../tmp/assets/examples/折木奉太郎.jpg"
)

for img in "${IMAGES[@]}"; do
  if [ ! -f "$img" ]; then
    echo "Warning: Image $img not found, skipping."
    continue
  fi
  
  base=$(basename "$img")
  name="${base%.*}"
  
  echo "Processing $name..."
  for fmt in "${FORMATS[@]}"; do
    echo "  -> Format: $fmt"
    ./target/release/vectomancy "$img" --format "$fmt" --mode spline --output "$OUT_DIR/${name}_$fmt"
  done
done

echo "All generation done."
