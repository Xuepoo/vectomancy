#!/bin/bash
OUTPUT_DIR="/tmp/agents/vectomancy/native_mass_test_output"
mkdir -p "$OUTPUT_DIR"

shopt -s nullglob
IMAGES=(/home/fuyu/Pictures/Wallpapers/Origin/*.jpg /home/fuyu/Pictures/Wallpapers/Origin/*.png)

for i in {0..29}; do
    IMAGE="${IMAGES[$i]}"
    if [ -f "$IMAGE" ]; then
        BASENAME=$(basename "$IMAGE")
        NAME="${BASENAME%.*}"
        
        echo "Processing [$((i+1))/30]: $BASENAME"
        
        cargo run --release -- "$IMAGE" --output "$OUTPUT_DIR/${NAME}_native.png" --format png --color --bg-transparent --mode spline --chaikin-iters 1 --tolerance 0.2 > /dev/null 2>&1
    fi
done
echo "Mass test complete. Output saved to $OUTPUT_DIR"