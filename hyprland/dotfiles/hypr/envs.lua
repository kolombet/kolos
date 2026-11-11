-- See https://wiki.hypr.land/Configuring/Advanced-and-Cool/Environment-variables/

hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
hl.env("XCURSOR_THEME", "Bibata-Modern-Classic")

-- Retroid Pocket 6 (pocknix) only: run KDE/Kirigami apps the way Plasma Mobile does (the same vars
-- startplasmamobile exports) - mobile QQC2 Breeze style + forced Kirigami tablet mode give
-- kinetic touch flicking in lists/pages instead of a touch drag acting like a mouse drag. Gated on
-- the device-tree model so other machines keep the desktop layout.
local function dt_model()
    local f = io.open("/sys/firmware/devicetree/base/model")
    if not f then return "" end
    local m = f:read("*a") or ""
    f:close()
    return m
end
kde_mobile_env = { "QT_QPA_PLATFORMTHEME", "QT_QUICK_CONTROLS_STYLE", "QT_QUICK_CONTROLS_MOBILE",
    "KDE_KIRIGAMI_TABLET_MODE", "PLASMA_PLATFORM", "PLASMA_INTEGRATION_USE_PORTAL" }
kde_mobile = dt_model():find("Retroid Pocket 6", 1, true) ~= nil
if kde_mobile then
    hl.env("QT_QPA_PLATFORMTHEME", "KDE")
    hl.env("QT_QUICK_CONTROLS_STYLE", "org.kde.breeze")
    hl.env("QT_QUICK_CONTROLS_MOBILE", "true")
    hl.env("KDE_KIRIGAMI_TABLET_MODE", "1")
    hl.env("PLASMA_PLATFORM", "phone:handset")
    hl.env("PLASMA_INTEGRATION_USE_PORTAL", "1")
end
