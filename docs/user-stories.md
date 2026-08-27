# User stories

Product-intent notes for this Hyprland setup — the "why" behind individual
config decisions in `hyprland/dotfiles/hypr/`, distinct from `CLAUDE.md`
(the "how" for anyone, human or agent, editing the config) and
`docs/philosophy.md` (the "what for" — why this system exists at all).

## Terminal

**As** the user of this setup, **I want** a maximally stable, bug-free, and
lightweight terminal emulator, **so that** the terminal itself is never the
source of crashes, rendering glitches, or wasted resources.

Acceptance criteria:
- Exactly one terminal is configured and installed: `foot`. No other
  terminal (rio, kitty, alacritty) appears in config, keybinds, or package
  lists — see the "Default terminal" section in `CLAUDE.md` for what was
  removed and why.
- `foot` renders via `pixman` (CPU/software) with no EGL/GPU dependency,
  avoiding the `EGL_BAD_MATCH` crashes seen with GPU-accelerated terminals
  (kitty's GLFW backend) on virtualized GPU stacks.
- Background transparency (`alpha=0.92` in `foot.ini`) is native to the
  terminal, not a compositor-side `windowrule opacity` hack — no dependency
  on compositor window-rule syntax that can silently break across Hyprland
  versions.

## New terminal opens in the current directory

**As** a user coming from macOS, **I want** opening a new terminal window to
start in the same directory as the terminal I'm currently looking at (like
Terminal.app), **so that** I don't have to re-`cd` every time I open a new
window from wherever I'm already working.

Acceptance criteria:
- `Mod+RETURN` opens a new `foot` window whose shell starts in the working
  directory of the most-recently-focused terminal, not `$HOME` or wherever
  the compositor happened to start.
- If no terminal is currently open/focused, it falls back to `$HOME` rather
  than erroring.
- Detection is based on the *shell's* directory, not `foot`'s own process
  cwd (which never changes when you `cd` inside the shell) — see
  `terminal-cwd.sh` and the "Keybindings" section of `CLAUDE.md`.

## Night-time blue-light filter

**As** a user, **I want** good sleep, **so that** the display shifts to a
warm, low-blue-light color temperature in the evening instead of staying at
full daylight color all night.

Acceptance criteria:
- `hyprsunset` is installed and autostarted (`hl.exec_cmd("hyprsunset")` in
  `autostart.lua`), so the filter is active every session without manual
  intervention.
- `hyprsunset.conf` defines two profiles: `identity = true` (no tint) from
  07:00, and `temperature = 3000` (warm) from 20:00 — a simple day/night
  schedule, not a continuous gradual ramp.
- The `identity = true` daytime profile is required — without it,
  `hyprsunset` applies its own default tint even during the day, which
  defeats the point of having a schedule at all.

## Shell prompt

**As** a user coming from macOS, **I want** the terminal prompt to show my
current directory the way Terminal.app's default zsh prompt does, **so
that** I always know where I am without running `pwd`, and without pulling
in an external prompt framework just for this.

Acceptance criteria:
- `~/.zshrc` sets `PS1` to `hostname:path %` (cyan hostname, blue path,
  `~`-shortened home directory), matching macOS's default zsh prompt shape.
- Root gets a red `#` instead of `%`, per the standard zsh `%(!.#.%)` root
  indicator.
- No external prompt framework (powerlevel10k, starship, etc.) is required
  — the prompt is a few lines of plain zsh, consistent with the "Portable,
  self-contained restore" story below.
- `hyprland/shell/zshrc` is deployed to `~/.zshrc` by both `restore.sh` and
  `restore-safe.sh`, the same way `zshenv`/`zprofile` already are —
  `restore-safe.sh` backs up any pre-existing `~/.zshrc` first.

## Focused floating window raises to top

**As** a user, **I want** the window I just switched focus to (via keyboard)
to visually come to the front, **so that** keyboard-driven focus changes
behave the same as clicking a window — I always see the window I'm
currently interacting with, not one buried behind others.

Acceptance criteria:
- Every focus-movement key (`hl.dsp.focus`) — arrows, `U/I/P`, `H/J/K/L` —
  and `Mod+Tab` / `Mod+Shift+Tab` (`hl.dsp.window.cycle_next`) each raise
  the newly-focused window to the
  top of the floating stack, via a paired `hl.dsp.window.alter_zorder({
  mode = "top" })` bind on the same key.
- Achieved with plain Hyprland config (multiple `hl.bind()` calls sharing a
  key combo, which Hyprland fires in order) — no external script or daemon,
  consistent with the "Portable, self-contained restore" story.
- Only meaningful for floating windows; harmless no-op when the focused
  window is tiled (tiled windows don't overlap, so z-order is moot).

## Host-captured shortcuts don't get relied on alone

**As** a user running this desktop as a VM guest (see
`docs/emulator-testing.md`), **I want** Hyprland's `Mod`-key combos to
survive the host intercepting some of them before the VM ever sees them,
**so that** a keypress meant to move window focus doesn't instead hide,
close, or quit the whole VM window, or lock me out of the host machine —
and so losing one key to host capture doesn't lose the whole action.

Acceptance criteria:
- Confirmed host-captured on this setup and never bound to anything, with
  no redundant fallback either: `Mod+Q` (quits the VM application),
  `Mod+W` (closes the VM window), `Mod+O` (opens something host-side).
  `Mod+BackSpace` replaces `Mod+Q` for closing the focused window; `Mod+X`
  replaces `Mod+W` for the dwindle togglesplit bind.
- `Mod+H` (hides the VM window) and `Mod+L` (locks the host session) are
  *also* confirmed host-captured here, but unlike `Q`/`W`/`O` they're still
  bound — as part of the redundant focus-movement scheme below, where a
  working arrow-key fallback exists alongside them.
- Window-focus movement is bound **three times over** — arrow keys
  (`Mod+Left/Right/Up/Down`), `Mod+U/I/P` (left/down/right), and
  `Mod+H/J/K/L` (left/down/up/right) all trigger the same four focus
  actions. Arrow keys are the only scheme confirmed safe on this host;
  `Mod+H`/`Mod+L` specifically are known-captured here too (see above) and
  are bound anyway, for muscle memory and for other hosts/deployments
  (e.g. bare-metal, no host above Hyprland to intercept anything) where
  they may work fine even though they don't here. `Mod+O` was dropped from
  the `U/I/O/P` set entirely, rather than kept as a dead redundant bind,
  once it turned out captured too (see `hyprland/dotfiles/hypr/bindings.lua`'s
  `focusKeys` table).
- The dwindle pseudo-tile toggle moved from `Mod+P` to `Mod+S`, since `P`
  is now claimed by right-focus and `Mod+K` (also claimed, by up-focus in
  the `H/J/K/L` scheme) was the other candidate already in use.
- Before wiring in any new global bind, check it against the confirmed-bad
  list (`Q`/`W`/`O`/`H`/`L`) — and if the action matters enough to want a
  guarantee, give it a redundant second binding the way focus movement has,
  rather than assuming a single key will reliably reach Hyprland on every
  host.

## Window resize

**As** a user, **I want** a dedicated, keyboard-only way to resize the
focused window, **so that** I don't need the mouse for fine window
adjustments.

Acceptance criteria:
- `Mod+R` enters a `resize` submap.
- Inside the submap: `H`/`L`/`K`/`J` resize left/right/up/down; `Return` or
  `Escape` exits back to normal binds.
- `Mod+R` is bound to nothing else. (It previously double-bound to launching
  `$apps`, which silently won over the resize submap — see `CLAUDE.md`.)

## Keyboard-only window move

**As** a user, **I want** a dedicated, keyboard-only way to move the
focused floating window, **so that** I don't need the mouse when it's
unavailable (or just don't want to reach for it).

Acceptance criteria:
- `Mod+M` enters a `move` submap, mirroring the existing `Mod+R` resize
  submap.
- Inside the submap: `H`/`L`/`K`/`J` move the window left/right/up/down in
  50px steps via `hl.dsp.window.move({ x, y, relative = true })` (parallel
  to `window.resize` in the `Mod+R` submap); `Return` or `Escape` exits back
  to normal binds.
- Freed up `Mod+M` by removing a duplicate bind (`Mod+M` → `firefox`), which
  was redundant with `Mod+B` → `$browser`.

## Tile / float mode

**As** a user, **I want** new windows to open floating by default, with a
one-key way to snap an entire workspace into tiled mode when I want it,
**so that** I get predictable window placement without being forced into
strict tiling.

Acceptance criteria:
- New windows open floating (not tiled), at a 960x540 centered starting
  size — the default state is floating, not tiled.
- `Mod+F` floats every window on the current workspace, restoring each
  window's previously saved position/size if one was captured by a prior
  `Mod+T`.
- `Mod+T` tiles every window on the current workspace, saving each window's
  current floating position/size first so `Mod+F` can restore it later.
- `Mod+V` (built-in `togglefloating`) toggles a single focused window
  independent of the workspace-wide `Mod+T`/`Mod+F` scripts, for one-off
  exceptions.

## Portable, self-contained restore

**As** the user, **I want** everything needed to reproduce my convenient
desktop setup to live in this repo, **so that** I can restore it effortlessly
on any Arch machine, or bootstrap it from scratch on a plain Arch CLI
install — with no dependency on Manjaro or any other downstream distro.

Acceptance criteria:
- A fresh Arch install (via `install.sh` → `0-preinstall.sh` /
  `1-setup.sh` / `2-user.sh` / `3-post-setup.sh`) or a lay-on-top install
  (via `hyprland/restore.sh` / `restore-safe.sh`) produces a fully working
  desktop using only packages available on vanilla Arch/Arch ARM — no
  Manjaro-only packages or paths.
- Every config file the desktop actually reads at runtime is either
  version-controlled in this repo (including machine-specific files like
  `monitors.lua`, which ships a generic default meant to be hand-edited in
  place rather than gitignored — see "Hyprland monitor config" in
  `CLAUDE.md`) or generated by a script in this repo. No config may silently
  `include=`/`require()` a path that only exists because it was inherited
  from a different distro's install.
