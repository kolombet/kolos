local mainMod = "SUPER"

hl.bind(mainMod .. " + Return", hl.dsp.exec_cmd(terminal .. " --working-directory=\"$(~/.config/hypr/terminal-cwd.sh)\""))
hl.bind(mainMod .. " + B", hl.dsp.exec_cmd(browser))
hl.bind(mainMod .. " + E", hl.dsp.exec_cmd(editor))
hl.bind(mainMod .. " + Z", hl.dsp.exec_cmd("zed"))
hl.bind(mainMod .. " + Y", hl.dsp.exec_cmd(fm))
hl.bind(mainMod .. " + BackSpace", hl.dsp.window.close())
hl.bind(mainMod .. " + F", hl.dsp.window.float({ action = "toggle" }))
hl.bind(mainMod .. " + T", hl.dsp.exec_cmd("~/.config/hypr/tile.sh"))
hl.bind(mainMod .. " + D", hl.dsp.exec_cmd(menu))
hl.bind(mainMod .. " + S", hl.dsp.window.pseudo()) -- dwindle
hl.bind(mainMod .. " + X", hl.dsp.layout("togglesplit")) -- dwindle only
hl.bind(mainMod .. " + grave", hl.dsp.exec_cmd("pkill -x kolos-keys || kolos-keys")) -- toggle cheatsheet (keys/)
hl.bind(mainMod .. " + slash", hl.dsp.exec_cmd("pkill -x kolos-keys || kolos-keys")) -- same, fallback if the host captures Mod+grave
-- Bound to the XF86 keysym, not "F1": hid_apple's fnmode=1 makes the bare
-- key send XF86MonBrightnessDown unconditionally, Super held or not, so
-- "SUPER + F1" would never fire. See CLAUDE.md.
hl.bind(mainMod .. " + XF86MonBrightnessDown", hl.dsp.exec_cmd("pkill -x kolos-keys || kolos-keys")) -- same, F1 = traditional help key
hl.bind(mainMod .. " + G", hl.dsp.window.fullscreen())

hl.bind(mainMod .. " + A", hl.dsp.workspace.toggle_special("quake")) -- drop-down terminal, see windows.lua
hl.bind(mainMod .. " + F12", hl.dsp.workspace.toggle_special("quake")) -- same, F12 = Guake/Yakuake key

hl.bind(mainMod .. " + Tab", hl.dsp.window.cycle_next())
hl.bind(mainMod .. " + Tab", hl.dsp.window.alter_zorder({ mode = "top" }))
hl.bind(mainMod .. " + SHIFT + Tab", hl.dsp.window.cycle_next({ previous = true }))
hl.bind(mainMod .. " + SHIFT + Tab", hl.dsp.window.alter_zorder({ mode = "top" }))

-- Move focus - three redundant key schemes mapped to the same four
-- directions, since this host intercepts some Mod+<letter> combos and
-- which ones varies by host/keyboard (Mod+O turned out captured too, on
-- top of the originally-reported Mod+Q/W/H/L - see "Host-captured
-- shortcuts never get bound" in docs/user-stories.md, updated to match).
-- Arrow keys are the safe baseline; U/I/P and H/J/K/L are kept as extra
-- muscle-memory options, accepting that any given one of them may turn out
-- to be captured on a given host and simply not fire.
local focusKeys = {
    { key = "Left",  direction = "left" },
    { key = "Right", direction = "right" },
    { key = "Up",    direction = "up" },
    { key = "Down",  direction = "down" },
    { key = "U", direction = "left" },
    { key = "P", direction = "right" },
    { key = "I", direction = "down" },
    { key = "H", direction = "left" },
    { key = "L", direction = "right" },
    { key = "J", direction = "down" },
    { key = "K", direction = "up" },
}

for _, f in ipairs(focusKeys) do
    hl.bind(mainMod .. " + " .. f.key, hl.dsp.focus({ direction = f.direction }))
    -- Raise the newly-focused floating window to the top of the stack -
    -- movefocus doesn't restack on its own (unlike a mouse click, which
    -- raises automatically). Two hl.bind() calls on the same combo both
    -- fire, same trick the old hyprlang config relied on with two `bind =`
    -- lines on one key.
    hl.bind(mainMod .. " + " .. f.key, hl.dsp.window.alter_zorder({ mode = "top" }))
end

-- Switch workspaces with mainMod + [0-9]
-- Move active window to a workspace with mainMod + SHIFT + [0-9]
for i = 1, 10 do
    local key = i % 10 -- 10 maps to key 0
    hl.bind(mainMod .. " + " .. key, hl.dsp.focus({ workspace = i }))
    hl.bind(mainMod .. " + SHIFT + " .. key, hl.dsp.window.move({ workspace = i }))
end

-- Scroll through existing workspaces with mainMod + scroll (disabled)
-- hl.bind(mainMod .. " + mouse_down", hl.dsp.focus({ workspace = "e+1" }))
-- hl.bind(mainMod .. " + mouse_up", hl.dsp.focus({ workspace = "e-1" }))

hl.bind(mainMod .. " + R", hl.dsp.exec_cmd("~/.config/hypr/float.sh"))

-- N was freed when the nvim launcher bind was dropped.
hl.bind(mainMod .. " + N", hl.dsp.submap("resize"))

hl.define_submap("resize", function()
    hl.bind("H", hl.dsp.window.resize({ x = -50, y = 0, relative = true }))
    hl.bind("L", hl.dsp.window.resize({ x = 50, y = 0, relative = true }))
    hl.bind("K", hl.dsp.window.resize({ x = 0, y = -50, relative = true }))
    hl.bind("J", hl.dsp.window.resize({ x = 0, y = 50, relative = true }))
    hl.bind("Return", hl.dsp.submap("reset"))
    hl.bind("Escape", hl.dsp.submap("reset"))
end)

hl.bind(mainMod .. " + M", hl.dsp.submap("move"))

hl.define_submap("move", function()
    hl.bind("H", hl.dsp.window.move({ x = -50, y = 0, relative = true }))
    hl.bind("L", hl.dsp.window.move({ x = 50, y = 0, relative = true }))
    hl.bind("K", hl.dsp.window.move({ x = 0, y = -50, relative = true }))
    hl.bind("J", hl.dsp.window.move({ x = 0, y = 50, relative = true }))
    hl.bind("Return", hl.dsp.submap("reset"))
    hl.bind("Escape", hl.dsp.submap("reset"))
end)

-- Move/resize windows with mainMod + LMB/RMB and dragging
hl.bind(mainMod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
hl.bind(mainMod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

-- Top-row media keys, bare (no Fn) - see "Function keys as media keys
-- (macOS-style)" in CLAUDE.md for why this needs hid_apple's fnmode=1 to
-- send XF86* keysyms here instead of literal F1-F12.
hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("brightnessctl set 5%-"), { repeating = true })
hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("brightnessctl set +5%"), { repeating = true })
hl.bind("XF86KbdBrightnessDown", hl.dsp.exec_cmd("brightnessctl --device='smc::kbd_backlight' set 10%-"), { repeating = true })
hl.bind("XF86KbdBrightnessUp", hl.dsp.exec_cmd("brightnessctl --device='smc::kbd_backlight' set +10%"), { repeating = true })
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"))
-- The audible range isn't the same fraction of PipeWire's volume scale on
-- every output (the Sony wireless USB headset's amp maxes out by ~10%; the
-- built-in speaker/headphone jack uses the full range) - volume.sh picks
-- the right cap per sink. kolos-bar's stats.rs volume() mirrors the same
-- per-sink logic to rescale its displayed VOL percent to match.
hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("~/.config/hypr/volume.sh down"), { repeating = true })
hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("~/.config/hypr/volume.sh up"), { repeating = true })
