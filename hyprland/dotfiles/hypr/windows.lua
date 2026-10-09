-- See https://wiki.hypr.land/Configuring/Basics/Window-Rules/ for more

-- New windows open floating by default, not tiled
hl.window_rule({
    name = "float-by-default",
    match = { class = ".*" },
    float = true,
    size = "960 540",
    center = true,
})

hl.window_rule({
    name = "suppress-maximize",
    match = { class = ".*" },
    suppress_event = "maximize", -- You'll probably like this.
})

-- Quake-style drop-down terminal: lives on the special workspace "quake",
-- toggled by Mod+A / Mod+F12 (bindings.lua). The first toggle creates the
-- workspace, and on_created_empty launches foot into it; later toggles just
-- hide/show that same window, so the shell keeps its state. Declared after
-- float-by-default so its size/position win (setting `center = false` here
-- would re-center it instead of turning centering off).
hl.workspace_rule({
    workspace = "special:quake",
    -- pad: the window's top 12px are off-screen (quake rule below), so the
    -- text needs more than that above it to stay fully visible.
    on_created_empty = terminal .. " --app-id=quake -o pad=8x16",
})

hl.window_rule({
    name = "quake",
    match = { class = "quake" },
    float = true,
    -- 5% padding left and right. Hyprland can't round individual corners,
    -- so the window starts 12px above the screen (rounding 10 + border 2,
    -- looknfeel.lua) and is 12px taller: its rounded top corners and top
    -- border sit off-screen, the visible top edge is square.
    size = "monitor_w*0.9 monitor_h*0.45+12",
    move = "monitor_w*0.05 -12",
})
