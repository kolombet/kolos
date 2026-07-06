#!/bin/bash
WORKSPACE=$(hyprctl activeworkspace -j | jq '.id')
STATEFILE="/tmp/hypr_float_ws_$WORKSPACE"

# Float all tiled windows
ADDRS=$(hyprctl clients -j | jq -r ".[] | select(.workspace.id == $WORKSPACE and .floating == false) | .address")
BATCH=""
for addr in $ADDRS; do
    BATCH+="dispatch togglefloating address:$addr;"
done
[ -n "$BATCH" ] && hyprctl --batch "$BATCH"

# Restore saved positions if available
if [ -f "$STATEFILE" ]; then
    BATCH=""
    while read -r win; do
        addr=$(echo "$win" | jq -r '.address')
        x=$(echo "$win" | jq -r '.x')
        y=$(echo "$win" | jq -r '.y')
        w=$(echo "$win" | jq -r '.w')
        h=$(echo "$win" | jq -r '.h')
        BATCH+="dispatch movewindowpixel exact $x $y,address:$addr;"
        BATCH+="dispatch resizewindowpixel exact $w $h,address:$addr;"
    done < "$STATEFILE"
    [ -n "$BATCH" ] && hyprctl --batch "$BATCH"
fi
