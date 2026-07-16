# kolos

Personal dotfiles / Arch install scripts (fork of ChrisTitusTech/ArchTitus), plus a bundled `editor/` app (Paperling, Tauri+Svelte).

See `docs/user-stories.md` for the product-intent reasoning behind terminal
choice, window resize, and tile/float behavior — this file covers the "how",
that one covers the "why". See `docs/emulator-testing.md` for running the
full `install.sh` path in a VM instead of on real hardware.

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

## Suspend-on-idle is laptop-only

`hypridle.conf`'s 10-minute idle listener runs `systemctl suspend` — correct
default for laptop hardware, but wrong for a QEMU/UTM VM: guest suspend
(s2idle) never resumes there, it hangs until the host hard-resets the VM.
This is machine-specific like `monitors.conf`, but isn't templated the same
way since most deployments of this repo are laptops. On a VM, after
deploying: remove the `systemctl suspend` `listener` block from
`~/.config/hypr/hypridle.conf` and `systemctl mask sleep.target
suspend.target hibernate.target hybrid-sleep.target` as a backstop against
anything else triggering suspend.

## Deployment

`2-user.sh` does `cp -r hyprland/dotfiles/* ~/.config/` — dotfiles are
copied, not symlinked. Since `monitors.conf` is gitignored, a fresh clone
won't have one; it must be created manually from the `.example` file after
running setup.

## Hyprland config layout

`hyprland.conf` is just `source =` lines — edit the sub-files, not it:
`monitors.conf` (gitignored, per-machine), `envs.conf`, `programs.conf`,
`looknfeel.conf`, `input.conf`, `autostart.conf`, `bindings.conf`,
`windows.conf`.

## Focus-raise on keyboard navigation

`bindings.conf` pairs every focus-changing bind (`movefocus` on
`Mod+H/L/K/J`, `cyclenext` on `Mod+Tab`/`Mod+Shift+Tab`) with a second
`bind =` line on the same key combo running `alterzorder, top`. Hyprland
fires multiple binds sharing a key in the order they're declared, so the
`alterzorder` line must come after the focus-changing one. This exists
because Hyprland only auto-raises a floating window's z-order on mouse
click, not on keyboard-driven focus changes — without this, moving focus
with the keyboard can leave the active window buried behind others.

## Keybindings

- `Mod+M` enters the `move` submap (`H`/`L`/`K`/`J` to move the focused
  floating window in 50px steps via `moveactive`, `Return`/`Escape` to
  exit) — same shape as the `resize` submap below, just with `moveactive`
  instead of `resizeactive`. `Mod+M` previously launched `firefox`
  redundantly with `Mod+B` → `$browser`; that duplicate bind was removed to
  free up the key.
- `Mod+R` enters the `resize` submap (`H`/`L`/`K`/`J` to resize, `Return`/
  `Escape` to exit) — defined at the bottom of `bindings.conf`. Don't bind
  `Mod+R` to anything else (e.g. an app launcher); it previously collided
  with `$apps` and the app-launch bind silently won because it was declared
  first.
- `Mod+T` runs `tile.sh` (tile all windows on the current workspace, saving
  their floating positions/sizes first) and `Mod+F` runs `float.sh` (float
  all windows on the current workspace, restoring saved positions/sizes if
  available). Both scripts live in `hyprland/dotfiles/hypr/` and need `jq`.
  Don't rebind `Mod+T`/`Mod+F` to app launchers (terminal/file manager) —
  that was the old behavior before these workspace-wide tile/float toggles
  were added.
- `Mod+RETURN` opens a new terminal in the same directory as the
  currently-focused terminal (macOS Terminal.app behavior), via
  `$terminal --working-directory="$(~/.config/hypr/terminal-cwd.sh)"`.
  `terminal-cwd.sh` finds the most-recently-focused `foot` window
  (`hyprctl clients -j`, sorted by `focusHistoryID`) and reads its direct
  shell child's `/proc/<pid>/cwd` — deliberately *not* a deeper recursive
  walk into whatever the shell is running, since that gets confused by
  processes that spawn their own large ephemeral subprocess trees (e.g. a
  nested Claude Code session). Needs `jq`.

## Window rules

New windows open floating (not tiled) by default, at a 960x540 centered
starting size — see the `windowrule = match:class .*, float on, size 960 540,
center on` line in `windows.conf`. This is why `tile.sh`/`float.sh` exist:
tiling isn't the default state, it's something you opt into per-workspace via
`Mod+T`.

## Night-time blue-light filter

`hyprsunset.conf` (sourced by nothing directly — read by the `hyprsunset`
binary itself, autostarted via `exec-once = hyprsunset` in
`autostart.conf`) defines two `profile` blocks: `identity = true` at
`07:00` and `temperature = 3000` at `20:00`. Don't drop the `identity`
profile — without it `hyprsunset` tints the screen by default even during
the day.

## Shell prompt

`hyprland/shell/zshrc` sets `PS1` to a macOS Terminal.app-style prompt
(`hostname:path %`, red `#` for root) via plain zsh `PROMPT_SUBST` — no
powerlevel10k or other framework. It's deployed to `~/.zshrc` by both
`restore.sh` and `restore-safe.sh`, alongside `zshenv`/`zprofile`. This is
separate from `2-user.sh` (the x86_64 disk-partitioning installer path),
which still clones `ChrisTitusTech/zsh` + `powerlevel10k` — that's a
different, heavier-weight product and out of scope here.

## Default terminal

`foot` is the only terminal — `$terminal = foot` in `programs.conf`. Rio,
kitty, and alacritty were removed (config, keybinds, and package-list
entries) since only foot was ever actually installed/used; rio in particular
was referenced in configs (`$terminal`, windowrules) but never installed
anywhere. Don't reintroduce another terminal without removing the old one
everywhere it's referenced: `programs.conf`, `bindings.conf` (`Mod+Y` opens
yazi via `foot yazi`), `input.conf` (scroll_touchpad windowrule targets
`match:class foot`), and the package lists in `1-setup.sh`,
`hyprland/restore.sh`, `hyprland/restore-safe.sh`.

Background transparency is set natively in `dotfiles/foot/foot.ini` via
`[colors-dark]` → `alpha=0.92` (not a compositor `windowrule = ... opacity`
— that dims the text too, not just the background). Note: this foot version
deprecated the old `[colors]` section name in favor of `[colors-dark]` /
`[colors-light]`; `foot --check-config` warns if you use the old name.

## Hyprland 0.55+ windowrule syntax

Per-window fields like touchpad scroll speed use `scroll_touchpad` (with an
underscore — not `scrolltouchpad`), and the selector must come *first*:
`windowrule = match:class foot, scroll_touchpad 1.5`. The legacy
`windowrule = scroll_touchpad 1.5, class:(foot)` form errors with "invalid
field class:(foot): missing a value" on this Hyprland version.
