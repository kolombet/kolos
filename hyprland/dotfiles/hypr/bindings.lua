local mainMod = "SUPER"

hl.bind(mainMod .. " + Return", hl.dsp.exec_cmd(terminal .. " --working-directory=\"$(~/.config/hypr/terminal-cwd.sh)\""))
hl.bind(mainMod .. " + B", hl.dsp.exec_cmd(browser))
hl.bind(mainMod .. " + E", hl.dsp.exec_cmd(editor))
hl.bind(mainMod .. " + Z", hl.dsp.exec_cmd("zed"))
hl.bind(mainMod .. " + N", hl.dsp.exec_cmd("nvim"))
hl.bind(mainMod .. " + BackSpace", hl.dsp.window.close())
hl.bind(mainMod .. " + F", hl.dsp.exec_cmd("~/.config/hypr/float.sh"))
hl.bind(mainMod .. " + T", hl.dsp.exec_cmd("~/.config/hypr/tile.sh"))
hl.bind(mainMod .. " + V", hl.dsp.window.float({ action = "toggle" }))
hl.bind(mainMod .. " + D", hl.dsp.exec_cmd(menu))
hl.bind(mainMod .. " + S", hl.dsp.window.pseudo()) -- dwindle
hl.bind(mainMod .. " + X", hl.dsp.layout("togglesplit")) -- dwindle only
hl.bind(mainMod .. " + grave", hl.dsp.exec_cmd("pkill -12 nwg-wrapper")) -- toggle cheatsheet

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

hl.bind(mainMod .. " + R", hl.dsp.submap("resize"))

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
