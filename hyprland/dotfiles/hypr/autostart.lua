-- See https://wiki.hypr.land/Configuring/Basics/Autostart/
-- hyprland.start fires once per session (not on `hyprctl reload`), matching
-- the old exec-once semantics.
hl.on("hyprland.start", function()
    hl.exec_cmd("waybar & ~/.config/hypr/randomwallpaper.sh & hyprctl setcursor Bibata-Modern-Classic 24")
    -- Uncomment to autostart the NetworkManager tray applet:
    -- hl.exec_cmd("nm-applet &")

    hl.exec_cmd("hypridle")
    hl.exec_cmd("hyprsunset")
end)
