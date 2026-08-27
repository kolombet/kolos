-- https://wiki.hypr.land/Configuring/Basics/Variables/
hl.config({
    xwayland = {
        force_zero_scaling = true,
    },
})

-- Waybar blur
hl.layer_rule({
    name = "waybar_blur",
    match = { namespace = "waybar" },
    blur = true,
    blur_popups = true,
    ignore_alpha = 0.2,
})

hl.config({
    general = {
        gaps_in = 2,
        gaps_out = 5,

        border_size = 2,

        col = {
            active_border = { colors = { "rgba(33ccffee)", "rgba(00ff99ee)" }, angle = 45 },
            inactive_border = "rgba(595959aa)",
        },

        -- Set to true to enable resizing windows by clicking and dragging on borders and gaps
        resize_on_border = false,

        -- Please see https://wiki.hypr.land/Configuring/Tearing/ before you turn this on
        allow_tearing = false,

        layout = "dwindle",
    },

    decoration = {
        rounding = 10,

        -- Change transparency of focused and unfocused windows
        active_opacity = 1.0,
        inactive_opacity = 1.0,

        blur = {
            enabled = true,
            size = 3,
            passes = 1,

            vibrancy = 0.1696,
        },
    },

    animations = {
        enabled = true,
    },
})

-- Default curves and animations, see https://wiki.hypr.land/Configuring/Advanced-and-Cool/Animations/
hl.curve("myBezier", { type = "bezier", points = { { 0.05, 0.9 }, { 0.1, 1.1 } } })

hl.animation({ leaf = "windows", enabled = true, speed = 2, bezier = "myBezier" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 2, bezier = "default", style = "popin 80%" })
hl.animation({ leaf = "border", enabled = true, speed = 2, bezier = "default" })
-- "borderangle" has no matching leaf name in the new animation engine as of
-- 0.56.1 (only "border" is exposed, checked against the installed type
-- stub) - dropped rather than guessed.
hl.animation({ leaf = "fade", enabled = true, speed = 2, bezier = "default" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 2, bezier = "default" })

-- See https://wiki.hypr.land/Configuring/Layouts/Dwindle-Layout/ for more
hl.config({
    dwindle = {
        preserve_split = true, -- You probably want this
    },
})
-- "pseudotile" (the old master switch for pseudotiling as a config default)
-- has no matching dwindle.* key in the installed type stub - dropped rather
-- than guessed. The Mod+P bind (bindings.lua) still toggles it at runtime.

-- See https://wiki.hypr.land/Configuring/Layouts/Master-Layout/ for more
hl.config({
    master = {
        new_status = "master",
    },
})

hl.config({
    misc = {
        force_default_wallpaper = 0, -- Set to 0 or 1 to disable the anime mascot wallpapers
        disable_hyprland_logo = false,
        disable_splash_rendering = true,
    },
})
