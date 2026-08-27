-- https://wiki.hypr.land/Configuring/Basics/Variables/
hl.config({
    input = {
        kb_layout = "us,ru",
        kb_variant = "",
        kb_model = "",
        kb_options = "grp:caps_toggle",
        kb_rules = "",

        follow_mouse = 1,

        sensitivity = 0, -- -1.0 - 1.0, 0 means no modification.

        repeat_rate = 40,
        repeat_delay = 600,

        touchpad = {
            natural_scroll = false,

            -- Use two-finger clicks for right-click instead of lower-right corner
            clickfinger_behavior = true,

            -- Control the speed of your scrolling
            scroll_factor = 0.4,
        },

        -- macOS-like mouse acceleration
        accel_profile = "adaptive",
    },
})

-- Example per-device config
-- See https://wiki.hypr.land/Configuring/Keywords/#per-device-input-configs for more
hl.device({
    name = "epic-mouse-v1",
    sensitivity = -0.5,
})

-- Scroll nicely in the terminal
hl.window_rule({
    name = "foot-scroll-speed",
    match = { class = "foot" },
    scroll_touchpad = 1.5,
})
