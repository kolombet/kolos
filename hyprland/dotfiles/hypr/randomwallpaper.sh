#!/bin/bash
WALLDIR="$HOME/.config/hypr/wallpapers"

mapfile -t walls < <(find "$WALLDIR" -type f \( -name '*.jpg' -o -name '*.png' -o -name '*.jpeg' -o -name '*.jxl' -o -name '*.webp' \) | shuf)

mapfile -t monitors < <(hyprctl monitors -j | python3 -c "import sys,json; [print(m['name']) for m in json.load(sys.stdin)]")

killall swaybg 2>/dev/null
sleep 0.3

args=()
for i in "${!monitors[@]}"; do
    wp="${walls[$((i % ${#walls[@]}))]}"
    args+=(-o "${monitors[$i]}" -i "$wp" -m fill)
done

swaybg "${args[@]}" &disown
