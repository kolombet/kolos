#!/bin/bash
WORKSPACE=$(hyprctl activeworkspace -j | jq '.id')
STATEFILE="/tmp/hypr_float_ws_$WORKSPACE"

# Float all tiled windows
ADDRS=$(hyprctl clients -j | jq -r ".[] | select(.workspace.id == $WORKSPACE and .floating == false) | .address")
LUA=""
for addr in $ADDRS; do
    LUA+="hl.dispatch(hl.dsp.window.float({ action = 'toggle', window = 'address:$addr' }));"
done
[ -n "$LUA" ] && hyprctl repl "$LUA return 'ok'"

# Restore saved positions if available
if [ -f "$STATEFILE" ]; then
    LUA=""
    while read -r win; do
        addr=$(echo "$win" | jq -r '.address')
        x=$(echo "$win" | jq -r '.x')
        y=$(echo "$win" | jq -r '.y')
        w=$(echo "$win" | jq -r '.w')
        h=$(echo "$win" | jq -r '.h')
        LUA+="hl.dispatch(hl.dsp.window.move({ x = $x, y = $y, exact = true, window = 'address:$addr' }));"
        LUA+="hl.dispatch(hl.dsp.window.resize({ x = $w, y = $h, exact = true, window = 'address:$addr' }));"
    done < "$STATEFILE"
    [ -n "$LUA" ] && hyprctl repl "$LUA return 'ok'"
fi
