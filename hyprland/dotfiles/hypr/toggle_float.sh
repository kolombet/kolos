#!/bin/bash
WORKSPACE=$(hyprctl activeworkspace -j | jq '.id')

# Single snapshot of all window state
SNAPSHOT=$(hyprctl clients -j | jq -c ".[] | select(.workspace.id == $WORKSPACE) | {address, x: .at[0], y: .at[1], w: .size[0], h: .size[1]}")

# Build batch command: toggle all, then restore all
BATCH=""
while read -r win; do
    addr=$(echo "$win" | jq -r '.address')
    BATCH+="dispatch togglefloating address:$addr;"
done <<< "$SNAPSHOT"

while read -r win; do
    addr=$(echo "$win" | jq -r '.address')
    x=$(echo "$win" | jq -r '.x')
    y=$(echo "$win" | jq -r '.y')
    w=$(echo "$win" | jq -r '.w')
    h=$(echo "$win" | jq -r '.h')
    BATCH+="dispatch movewindowpixel exact $x $y,address:$addr;"
    BATCH+="dispatch resizewindowpixel exact $w $h,address:$addr;"
done <<< "$SNAPSHOT"

hyprctl --batch "$BATCH"
