-- See https://wiki.hypr.land/Configuring/Advanced-and-Cool/Environment-variables/

hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
-- Bibata where it's installed (AUR-only), else the first other theme present (Breeze on a
-- KDE/Pocknix system): a theme that isn't installed leaves no pointer image to load.
local function has_cursor_theme(name)
    for _, dir in ipairs({ os.getenv("HOME") .. "/.local/share/icons/", "/usr/share/icons/" }) do
        local f = io.open(dir .. name .. "/cursors/left_ptr")
        if f then f:close() return true end
    end
    return false
end
cursor_theme = "Adwaita"
for _, name in ipairs({ "Bibata-Modern-Classic", "breeze_cursors" }) do
    if has_cursor_theme(name) then cursor_theme = name break end
end
hl.env("XCURSOR_THEME", cursor_theme)

-- kolos-bar, kolos-keys and kickoff live in ~/.local/bin; a session not started from a zsh
-- login (a display manager, Pocknix's desktop picker) never reads ~/.zshenv's PATH.
hl.env("PATH", os.getenv("HOME") .. "/.local/bin:" .. (os.getenv("PATH") or "/usr/bin"))
