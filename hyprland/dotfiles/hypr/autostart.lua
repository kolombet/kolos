-- See https://wiki.hypr.land/Configuring/Basics/Autostart/
-- hyprland.start fires once per session (not on `hyprctl reload`), matching
-- the old exec-once semantics.
hl.on("hyprland.start", function()
    hl.exec_cmd("kolos-bar & ~/.config/hypr/randomwallpaper.sh & hyprctl setcursor Bibata-Modern-Classic 24")
    -- Uncomment to autostart the NetworkManager tray applet:
    -- hl.exec_cmd("nm-applet &")

    -- Hand the RP6's Plasma Mobile env (envs.lua) to D-Bus/systemd-activated KDE apps too.
    if kde_mobile then
        hl.exec_cmd("dbus-update-activation-environment --systemd " .. table.concat(kde_mobile_env, " "))
    end
    hl.exec_cmd("hypridle")
    hl.exec_cmd("hyprsunset")
end)
