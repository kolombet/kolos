-- Per-machine monitor layout. Unlike the old monitors.conf (gitignored,
-- copied from monitors.conf.example per machine), this file is tracked
-- directly - it ships with a safe, generic default and gets hand-edited
-- in place on whichever machine needs something more specific.

hl.monitor({
    output = "",
    mode = "preferred",
    position = "auto",
    scale = 1,
})

-- Multi-monitor / HiDPI: replace the block above (don't just uncomment
-- these - edit output names/modes for the actual machine):
-- hl.monitor({ output = "DP-1", mode = "3840x2160@60", position = "0x0", scale = 2 })
-- hl.monitor({ output = "HDMI-A-1", mode = "1920x1080@60", position = "1920x0", scale = 1 })
--
-- Assign a workspace to a specific monitor:
-- hl.workspace_rule({ workspace = "5", monitor = "HDMI-A-1" })
