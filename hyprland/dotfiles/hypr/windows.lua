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
