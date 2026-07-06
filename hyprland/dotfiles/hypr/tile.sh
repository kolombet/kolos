#!/bin/bash
WORKSPACE=$(hyprctl activeworkspace -j | jq '.id')
STATEFILE="/tmp/hypr_float_ws_$WORKSPACE"

# Save current positions/sizes before tiling
hyprctl clients -j | jq -c ".[] | select(.workspace.id == $WORKSPACE) | {address, x: .at[0], y: .at[1], w: .size[0], h: .size[1]}" > "$STATEFILE"

# Tile all floating windows
BATCH=""
while read -r win; do
    addr=$(echo "$win" | jq -r '.address')
    floating=$(hyprctl clients -j | jq -r ".[] | select(.address == \"$addr\") | .floating")
    if [ "$floating" = "true" ]; then
        BATCH+="dispatch togglefloating address:$addr;"
    fi
done < "$STATEFILE"

[ -n "$BATCH" ] && hyprctl --batch "$BATCH"
