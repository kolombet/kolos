#!/bin/bash
WALLDIR="$HOME/.config/hypr/wallpapers"
CONF="$HOME/.config/hypr/hyprpaper.conf"

mapfile -t walls < <(find "$WALLDIR" -type f \( -name '*.jpg' -o -name '*.png' -o -name '*.jpeg' -o -name '*.jxl' -o -name '*.webp' \) | shuf)

monitors=($(hyprctl monitors -j | python3 -c "import sys,json; [print(m['name']) for m in json.load(sys.stdin)]"))

echo "splash = false" > "$CONF"
for i in "${!monitors[@]}"; do
    wp="${walls[$((i % ${#walls[@]}))]}"
    cat >> "$CONF" <<EOF
wallpaper {
    monitor = ${monitors[$i]}
    path = $wp
}

EOF
done

killall hyprpaper 2>/dev/null
sleep 0.3
hyprpaper &disown
