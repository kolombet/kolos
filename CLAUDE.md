# kolos

Personal dotfiles / Arch install scripts (fork of ChrisTitusTech/ArchTitus), plus a bundled `editor/` app (Paperling, Tauri+Svelte).

## Hyprland monitor config

`hyprland/dotfiles/hypr/hyprland.conf` does NOT hardcode monitor layout or
workspace-to-monitor bindings — those are machine-specific and vary between
setups (single virtual/HiDPI display vs. multi-monitor desktop).

Instead it has:

```
source = ~/.config/hypr/monitors.conf
```

`monitors.conf` is gitignored and must be created per-machine (copy
`hyprland/dotfiles/hypr/monitors.conf.example` and edit for the current
machine's monitor names/resolutions). Don't put `monitor=` or
`workspace = N, monitor:X` lines directly into `hyprland.conf` — add them to
the local `monitors.conf` instead.

## Deployment

`2-user.sh` does `cp -r hyprland/dotfiles/* ~/.config/` — dotfiles are
copied, not symlinked. Since `monitors.conf` is gitignored, a fresh clone
won't have one; it must be created manually from the `.example` file after
running setup.
