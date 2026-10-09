# kolos

Personal dotfiles / Arch install scripts, plus a bundled `editor/` app (Paperling, Tauri+Svelte).

See `docs/philosophy.md` for the "what for" — why this system exists and
what it's trying to be. See `docs/user-stories.md` for the product-intent
reasoning behind terminal choice, window resize, and tile/float behavior —
this file covers the "how", that one covers the "why" for individual
decisions. See `docs/emulator-testing.md` for running the full
`install.sh` path in a VM instead of on real hardware.

## Hyprland monitor config

`hyprland/dotfiles/hypr/monitors.lua` ships a single generic default —
`hl.monitor({ output = "", mode = "preferred", position = "auto", scale = 1
})` — meant to work on any machine out of the box. Unlike the old
`monitors.conf` (gitignored, copied per-machine from a `.example` file),
`monitors.lua` is tracked directly and gets **hand-edited in place** on
whichever machine needs something more specific (a VM's virtual output, a
multi-monitor desktop, HiDPI scaling). There's no example/gitignore
indirection anymore — see the commented-out `hl.monitor({...})` /
`hl.workspace_rule({...})` examples at the bottom of the file.

## Suspend-on-idle is laptop-only

`hypridle.conf`'s 10-minute idle listener runs `systemctl suspend` — correct
default for laptop hardware, but wrong for a QEMU/UTM VM: guest suspend
(s2idle) never resumes there, it hangs until the host hard-resets the VM.
This is machine-specific like `monitors.lua`, but isn't templated the same
way since most deployments of this repo are laptops. On a VM, after
deploying: remove the `systemctl suspend` `listener` block from
`~/.config/hypr/hypridle.conf` and `systemctl mask sleep.target
suspend.target hibernate.target hybrid-sleep.target` as a backstop against
anything else triggering suspend.

## Deployment

`2-user.sh` does `cp -r hyprland/dotfiles/* ~/.config/` — dotfiles are
copied, not symlinked. `monitors.lua` is tracked and deploys with its
generic default like every other file here; no manual per-machine setup
step is required before first login (though the monitor layout may need
hand-editing afterward — see "Hyprland monitor config" above).

## GUI sudo password prompt

Some commands need `sudo` but have no controlling TTY — a Hyprland keybind
launching a command directly, or a tool (e.g. an agent's shell tool)
invoking `sudo` non-interactively. Without an askpass helper, sudo just
fails with "a password is required" instead of prompting.

Three approaches were tried, in order:

- **`SUDO_ASKPASS` env var in `~/.zshenv`**, pointing at
  `lxqt-openssh-askpass`. Rejected — it only applies to processes that
  inherit that shell's environment, so it doesn't cover the motivating case
  (a keybind launching a program directly, with no zsh login shell in its
  ancestry). It also isn't the same mechanism sudo falls back to when
  nothing is set (see below), so having both configured at once invites
  confusion about which one actually fires.
- **System-wide `/etc/sudo.conf`** with `Path askpass
  /usr/local/bin/gui-askpass`, a script wrapping `zenity --password`. This
  applies to *any* process invoking `sudo` without a TTY regardless of
  shell or ancestry, which is the part that actually matters — but in
  practice it printed GTK theme-parser errors and a failed
  Vulkan-device-detection warning to stderr every time it fired, and pulls
  in a full toolkit (GTK3) for one modal.
- **`askpass/` (`kolos-askpass`) — the option actually used.** A small
  standalone Rust binary that draws its own Wayland layer-shell surface
  (via `smithay-client-toolkit` + `fontdue`, no GTK/Qt at all) instead of
  shelling out to anything. It's deliberately not a fork of `kickoff`
  (this repo's actual launcher) — kickoff's fuzzy search, clipboard,
  history, and config-file system are all dead weight for a password
  box — but it reuses kickoff's proven techniques for the parts that
  matter: `Anchor::all()` + `set_exclusive_zone(-1)` on an `Overlay`-layer
  surface (draws a small box manually centered within a full-output
  canvas, rather than hoping an unanchored surface centers itself),
  `get_keyboard_with_repeat` for key-repeat, and reading typed text via
  `KeyEvent.utf8` rather than hand-rolling keysym-to-Unicode translation.
  `argv[1]` is the prompt (sudo passes its own prompt text there); Enter
  prints the buffer to stdout and exits 0, Escape exits nonzero with no
  output.

Deployed by `restore.sh`/`restore-safe.sh`: install the `rust` package
(for `cargo`/`rustc`, build-time only), run `cargo build --release
--manifest-path askpass/Cargo.toml` as the invoking user (not `$PRIV` —
don't build as root), `install -Dm755` the resulting binary to
`/usr/local/bin/kolos-askpass`, then point `/etc/sudo.conf`'s `Path
askpass` at it — replacing an existing `Path askpass` line in place via
`sed` if one is already there (e.g. from the earlier zenity setup),
appending one otherwise. `1-setup.sh` (the separate x86_64
full-disk-installer path, out of scope otherwise — see "Deployment" above)
needs none of this, since it sets `%wheel ALL=(ALL) NOPASSWD: ALL` and
never prompts wheel members for a password at all.

## SSH key passphrase prompt

`kolos-askpass` also serves as `SSH_ASKPASS`. It follows the same protocol
ssh expects (prompt in `argv[1]`, passphrase on stdout, nonzero exit on
cancel), so no separate binary is needed.

Previously `SSH_AUTH_SOCK` pointed at `gcr-ssh-agent`
(`$XDG_RUNTIME_DIR/gcr/ssh`). That agent unlocks keys through
`gcr-prompter`, a GTK modal with no askpass hook, so any git-over-SSH from a
process without a TTY (an agent's shell tool, a keybind) popped up GTK
instead of the repo's own prompt. Replaced with OpenSSH's own `ssh-agent`
via its systemd user unit `ssh-agent.socket` (socket at
`$XDG_RUNTIME_DIR/ssh-agent.socket`):

- `shell/zshenv` exports `SSH_AUTH_SOCK` and
  `SSH_ASKPASS=/usr/local/bin/kolos-askpass`. `SSH_ASKPASS_REQUIRE` is left
  unset on purpose: ssh uses askpass only when there is no TTY (and
  `DISPLAY`/`WAYLAND_DISPLAY` is set, as it is under Hyprland) and keeps
  prompting in the terminal otherwise.
- `restore.sh`/`restore-safe.sh` disable `gcr-ssh-agent.{socket,service}`,
  enable `ssh-agent.socket` (user units, so only when not run as root), and
  prepend `AddKeysToAgent yes` to `~/.ssh/config` if it is missing, so the
  key is unlocked once per login session instead of on every push.

Trade-off: gcr could store the passphrase in the GNOME login keyring and
never ask again; with this setup it is asked once after each login.
`gnome-keyring`, `gcr`/`gcr-4` and `seahorse` were then removed entirely
(nothing used the keyring; `gh` keeps its token in `~/.config/gh/hosts.yml`).

## No Qt, no GTK

Nothing kolos installs may pull in Qt (`qt5-base`, `qt6-base`, KDE
Frameworks) or GTK (`gtk3`, `gtk4`, `libadwaita`), directly or as a
dependency. Apps are toolkit-free instead: COSMIC's iced-based apps
(`cosmic-files`), or small Rust layer-shell tools in this repo (`askpass/`,
`keys/`, `bar/`). It started as "No Qt" (one toolkit, GTK, because Firefox
needed it); once Chromium turned out to run without GTK, GTK went too.
Removed for this reason: `telegram-desktop` (use web.telegram.org),
`polkit-kde-agent`, `qt5ct`/`qt6ct`, Waybar, nwg-wrapper, pcmanfm,
spice-vdagent, zenity, gnome-keyring/seahorse, Firefox.

The browser, Brave Origin (Brave's stripped-down build: no AI, crypto, VPN,
Rewards or Tor; free on Linux), is the one exception on paper. Its package,
`brave-origin-bin` (AUR-only, maintained by Brave), lists `gtk3` as a hard
dependency, but the binary has no `libgtk` in its `NEEDED` entries. It
dlopen()s libgtk-3/libgtk-4 only if present, and without them it runs normally
(verified here: native Wayland window without `--ozone-platform` flags, no GTK
or Qt mapped in any process) and gets open/save dialogs from the file-chooser
portal (below). The bundled `libqt5_shim.so`/`libqt6_shim.so` link Qt but are
only loaded if Qt is installed. The package declares only `alsa-lib gtk3
libxss nss ttf-font`, yet the binary also links `at-spi2-core`, `cairo`,
`pango`, `libcups`, `libxcomposite`/`libxdamage`/`libxrandr`, `libxkbcommon`
and `mesa`, which the real `gtk3` would have pulled in and `nogtk3` doesn't. The
restore scripts install those explicitly; never remove them as orphans. It replaced Chromium, which behaves the same
way, for the stripped-down build; the cost is that it's AUR-only, so
`pacman -Syu` doesn't update it: rebuild `brave-origin-bin` by hand (or with an
AUR helper) to get browser security updates.
`hyprland/nogtk3/PKGBUILD` builds `nogtk3`, an empty package that
`provides`/`conflicts` the bare `gtk3` name, so pacman considers that
dependency met and won't reinstall GTK3 on `-Syu`. It only fakes the
unversioned name: a package depending on `gtk3>=...` or on the
`libgtk-3.so` soname still fails to install, which is the intended safety
net. `restore.sh`/`restore-safe.sh` build and install it (as the invoking
user, since makepkg refuses root) *before* building and installing
`brave-origin-bin` the same way (plain `makepkg` from the AUR git repo, no
AUR helper needed), so GTK3 is never installed in the first place. Run as
root, they skip both and install no browser. `2-user.sh` (x86_64 installer
path) installs it with yay and doesn't build `nogtk3`, so there it still
pulls in gtk3.

Before adding a package to any `PKGS` list, check its full dependency tree,
e.g. `pacman -Sp --print-format %n <pkg> | grep -E '^(qt|kf|gtk|libadwaita)'`
(prints nothing if clean; run it here, where neither toolkit is installed, so
nothing is hidden as already installed). Prefer toolkit-free alternatives,
e.g. `cosmic-files` (not `pcmanfm`/`pcmanfm-qt`), a Chromium-based browser
not Firefox/qutebrowser/Falkon. Not all
of COSMIC qualifies: `cosmic-settings-daemon` hard-depends on `qt6ct`
(Qt6) and `cosmic-settings` on `nm-connection-editor` (GTK3), so install
individual COSMIC apps, never the `cosmic` group. Many packages list Qt only
as an *optional* dependency (e.g. `cmake` for `cmake-gui`); that's fine,
don't install the optional part. Never set `QT_QPA_PLATFORMTHEME` or other
`QT_*` env vars.

Desktop portals follow the same rule. `xdg-desktop-portal-hyprland` needs
Qt6 (its screen-share picker) and `xdg-desktop-portal-gtk` needs GTK, so
neither is installed; `dotfiles/xdg-desktop-portal/hyprland-portals.conf`
routes only the file chooser to `xdg-desktop-portal-cosmic` (iced). Brave
uses that portal for open/save dialogs once GTK is gone. Consequence: no
screen-sharing portal (WebRTC screen share won't work) until a Qt-free one
exists.

`restore.sh`/`restore-safe.sh` end with a check that prints a warning, plus
the packages that required it, if `qt5-base`, `qt6-base`, `gtk3` or `gtk4`
ended up installed anyway (`pacman -Qq` lists real package names, so
`nogtk3` doesn't trip it).

## Xwayland is off

`looknfeel.lua` sets `xwayland = { enabled = false }`: everything here
(foot, Brave Origin, cosmic-files, the `bar/`/`keys/`/`askpass/` tools) is
native Wayland, and an idle Xwayland cost ~40 MiB. `xorg-xwayland` stays
installed only because `hyprland` hard-depends on it. Any X11-only app will
fail to start; re-enable it there rather than per-app. The setting only takes
effect on Hyprland startup, not on `hyprctl reload`.

## Keybinding cheat sheet

`keys/` (`kolos-keys`) replaced `nwg-wrapper` (GTK3, AUR-only, and pinned
to output `HDMI-A-1`, so it never showed on a VM's `Virtual-1`). Same
toolkit-free approach as `askpass/`: an `Overlay` layer-shell surface,
anchored top-right, sized to its content, with an empty input region so
clicks fall through, and no output given so it lands on the focused one.
It reuses askpass's text rendering via `#[path = "../../askpass/src/font.rs"]`
rather than a copy, so a font change there applies to both. It draws at the
output's integer scale like `bar/`: it starts at scale 1 (the surface isn't
on an output yet), then on `scale_factor_changed` recomputes its layout at
that scale, calls `set_buffer_scale`, and resizes itself if rounding moved
the logical size.

It is not a daemon: `Mod+grave` (plus `Mod+slash`, `Mod+F1` and `Mod+G`
as fallbacks in case the host captures `Mod+grave`, as macOS does with
Cmd+`) runs `pkill -x kolos-keys || kolos-keys`,
so pressing it starts the process (shown) or kills it (hidden). Keep the
binary name at most 15 characters, since `pkill -x` matches the kernel's
truncated process name. Content comes from `~/.config/hypr/keys.txt`
(deployed with the rest of `dotfiles/hypr/`): `# title` lines are section
headers, `label  keys` lines (split on the first 2+ spaces) are two-column
entries, an empty line is a small gap. **Update `keys.txt` whenever
`bindings.lua` changes**; the old nwg-wrapper sheet had drifted (it still
listed `Mod+Q`/`Mod+W`, both host-captured and unbound).

Built by `restore.sh`/`restore-safe.sh` as the invoking user and installed
to `~/.local/bin/kolos-keys` (no root needed, and `~/.local/bin` is on
Hyprland's `PATH` via `zshenv`). `2-user.sh` doesn't build it, so on that
path `Mod+grave` does nothing until it's built by hand.

## Drop-down terminal

`Mod+A` (fallback `Mod+F12`) toggles a Quake-style terminal. It's pure
Hyprland config, not a `bar/`-style Rust tool, since a terminal emulator
(PTY, VT parsing, scrollback, selection) is far beyond those: the special
workspace `special:quake` has `on_created_empty` launching
`foot --app-id=quake`, and the `quake` window rule in `windows.lua` sizes it
to 90% of the monitor width (5% padding each side), 45% height, flush with
the top edge. Hyprland can't round individual corners, so for square top
corners the window starts 12px above the screen (`rounding` 10 + `border_size`
2 from `looknfeel.lua`; update both if those change) and is 12px taller, and
foot gets `-o pad=8x16` so its first line isn't hidden up there too. On a
monitor with another one directly above, those 12px would show on that one. Toggling only hides or
shows the workspace, so the shell keeps its state; closing the shell makes
the next toggle start a fresh one. Don't add `center = false` to that rule:
setting it re-centers the window instead of overriding `float-by-default`'s
`center = true`. The drop-down motion is the `specialWorkspaceIn`/`specialWorkspaceOut`
animation leaves in `looknfeel.lua`, styles `slidevert top` and `slidevert
bottom`. The direction is where the incoming view slides in from, on exit too:
`bottom` on Out means the view behind comes up from below, pushing the
terminal up.
Special workspaces otherwise inherit the horizontal `workspaces` slide,
plain `slidevert` comes up from the bottom, and an `animation` on the workspace rule has no effect
on the special toggle. `kolos-bar` ignores special workspaces (id <= 0), so it
never appears in the taskbar.

## Status bar

`bar/` (`kolos-bar`) replaced Waybar (GTK3 via gtkmm3). It's the same
toolkit-free approach as `askpass/`/`keys/` and shares their `font.rs`: one
`Top` layer-shell surface per output, anchored bottom/left/right, with an
exclusive zone of its height, drawn into shm buffers at the output's integer
scale (`set_buffer_scale`, so text stays sharp at scale 2). Measured: about
6 MiB PSS, against ~43 MiB for Waybar.

- `hypr.rs` talks to Hyprland's sockets directly (no `hyprctl` processes):
  `j/monitors`, `j/workspaces`, `j/clients`, `j/activewindow`, `j/devices`
  for state, `dispatch <lua>` for clicks (the same Lua dispatcher syntax as
  `bindings.lua`), and the `.socket2.sock` event stream to know when to
  re-query. Title events are ignored on purpose: some windows (a spinner in a
  terminal title) retitle several times a second, and the bar doesn't show
  titles.
- `stats.rs` reads `/proc/stat`, `/proc/meminfo`,
  `/sys/class/power_supply` and `/sys/class/thermal` every 2 s; the bar
  redraws only when something it shows changed. Battery only appears when a
  `Battery` supply exists. CPU temperature comes from the `x86_pkg_temp`
  thermal zone specifically, not `thermal_zone0` — zone numbering isn't
  stable across machines, and on this one zone0 is `BAT0` (a battery
  sensor), which is why the old Waybar config (no explicit
  `thermal-zone`/`hwmon-path`, so it used zone0) quietly displayed battery
  temperature mislabeled as CPU temperature.
- Left: workspaces 1-5 always (Waybar's `persistent-workspaces`), plus any
  other workspace on that output; white + underline = active, gray =
  has windows, dark = empty. Then one entry per window on that output, by
  workspace, then in the order the bar first saw them (`window_order`), not
  Hyprland's `j/clients` order: that one is stacking order, and every focus
  keybind raises the window, so following it made the focused window jump
  to the end. Labeled by the last dot-segment of its class. Click focuses,
  middle-click closes, wheel (notches only, not touchpad) moves to the
  next/previous existing workspace.
- Right: `CPU <load>% <temp>°C`, memory, keyboard layout (first three
  letters of `active_keymap`: ENG, RUS), battery (`BAT <capacity>%`, `+`
  suffix while charging), two-line clock. Plain text, no icons: the *Mono*
  Nerd Font scales icons to one cell, unreadably small.

Not done yet, compared to Waybar: app icons in the taskbar (needs icon-theme
lookup plus PNG/SVG decoding), the tray (StatusNotifierItem over D-Bus;
nothing here uses it), volume (PipeWire is installed, but there's no
lightweight text protocol for it the way Hyprland's sockets give `hypr.rs` —
would mean shelling out to `wpctl` or writing a native PipeWire protocol
client) and the calendar tooltip on the clock. Look/feel constants (sizes,
colors from the old `style.css`) are at the top of `bar/src/main.rs`; there's
no config file.

Built by `restore.sh`/`restore-safe.sh` like `keys/` (to
`~/.local/bin/kolos-bar`), started from `autostart.lua`, blurred behind by
the `bar_blur` layer rule in `looknfeel.lua`. To restart it after a rebuild:
`pkill -x kolos-bar; kolos-bar &`.

## Hyprland config layout

As of Hyprland 0.55, the old hyprlang `.conf` syntax is deprecated in favor
of a Lua config, with removal planned "1-2 releases" out (see
https://hypr.land/news/26_lua/) — this repo has fully switched, there's no
`.conf` fallback kept around. `hyprland.lua` is just a list of
`require(...)` calls — edit the sub-files, not it: `monitors.lua`
(per-machine, see above), `envs.lua`, `programs.lua`, `looknfeel.lua`,
`input.lua`, `autostart.lua`, `bindings.lua`, `windows.lua`. If both a
`.lua` and a `.conf` version of a file exist, `.lua` wins — the choice is
made once at Hyprland startup, not on `hyprctl reload`.

Old hyprlang → new Lua, the shapes that come up most in this repo:

| Old hyprlang | New Lua |
|---|---|
| `source = ~/.config/hypr/foo.conf` | `require("foo")` |
| `$var = value` | a bare (non-`local`) global `var = "value"`, if other files need to read it via `require` — e.g. `programs.lua`'s `terminal`/`browser`/`menu`/etc. |
| `monitor=,mode,pos,scale` | `hl.monitor({ output = "...", mode = "...", position = "...", scale = ... })` |
| `env = NAME,value` | `hl.env("NAME", "value")` |
| `exec-once = cmd` | `hl.exec_cmd("cmd")` inside `hl.on("hyprland.start", function() ... end)` |
| `category { key = value }` | `hl.config({ category = { key = value } })`, nesting mirrors the old block nesting |
| `bind = MOD, KEY, dispatcher, arg` | `hl.bind("MOD + KEY", hl.dsp.<namespace>.<dispatcher>({ ...args }))` |
| `submap = name` / binds / `submap = reset` | `hl.define_submap("name", function() hl.bind(...) end)`, entered via `hl.dsp.submap("name")`, exited via `hl.dsp.submap("reset")` inside it |
| `windowrule = match:class foo, prop val` | `hl.window_rule({ match = { class = "foo" }, prop = val })` — property names (`float`, `size`, `center`, `suppress_event`, `scroll_touchpad`, …) carry over unchanged |
| `layerrule = match:namespace = x, blur 1` | `hl.layer_rule({ match = { namespace = "x" }, blur = true })` |

Two fields from the old `.conf` files have no equivalent in the installed
type stub (`/usr/share/hypr/stubs/hl.meta.lua`, autogenerated per Hyprland
build — the ground-truth source for what fields/dispatchers actually
exist) and were dropped rather than guessed: `animation = borderangle, ...`
(only a `border` leaf exists now) and `dwindle { pseudotile = true }` (no
matching `dwindle.*` key — the `Mod+P` bind still toggles it at runtime,
it just isn't settable as a startup default). Before adding a new Lua
construct, grep that stub file rather than assuming an old hyprlang field
name still applies — several of the ones above were confirmed by running
`hyprctl repl '<lua>'` against a live compositor rather than guessing from
the wiki.

`tile.sh`/`float.sh` had to change too: Hyprland 0.56+ broke the old
`hyprctl --batch "dispatch foo;dispatch bar;"` string form (dispatch now
expects a real Lua expression). Both scripts build a Lua chunk instead and
run it via `hyprctl repl "<chunk> return 'ok'"` — e.g.
`hl.dispatch(hl.dsp.window.float({ action = 'toggle', window = 'address:0x...' }))`.
If a script shells out to `hyprctl ... dispatch <name> <args>` and silently
stops working, this is why — Hyprland logs a Lua parse error but the
caller just sees a nonzero/ignored exit code.

## Focus-raise on keyboard navigation

`bindings.lua` pairs every focus-changing bind (`hl.dsp.focus({...})` on
each key in the `focusKeys` table — arrows, `U/I/P`, and `H/J/K/L` — plus
`hl.dsp.window.cycle_next(...)` on `Mod+Tab`/`Mod+Shift+Tab`) with a second
`hl.bind()` call on the same key combo
running `hl.dsp.window.alter_zorder({ mode = "top" })`. Hyprland fires
multiple binds sharing a key in the order they're declared, so the
`alter_zorder` call must come after the focus-changing one. This exists
because Hyprland only auto-raises a floating window's z-order on mouse
click, not on keyboard-driven focus changes — without this, moving focus
with the keyboard can leave the active window buried behind others.

## Keybindings

- `Mod+M` enters the `move` submap (`H`/`L`/`K`/`J` to move the focused
  floating window in 50px steps via `hl.dsp.window.move({ x, y, relative =
  true })`, `Return`/`Escape` to exit) — same shape as the `resize` submap
  below, just with `window.move` instead of `window.resize`. `Mod+M`
  previously launched `firefox` redundantly with `Mod+B` → `$browser`; that
  duplicate bind was removed to free up the key.
- `Mod+R` enters the `resize` submap (`H`/`L`/`K`/`J` to resize, `Return`/
  `Escape` to exit) — defined at the bottom of `bindings.lua`. Don't bind
  `Mod+R` to anything else (e.g. an app launcher); it previously collided
  with `$apps` and the app-launch bind silently won because it was declared
  first.
- `Mod+Q`, `Mod+W`, `Mod+O` are confirmed host-captured and never bound to
  anything, with no fallback — see "Host-captured shortcuts don't get
  relied on alone" in `docs/user-stories.md`. Closing the focused window is
  `Mod+BackSpace` (was `Mod+Q`); the dwindle togglesplit bind is `Mod+X`
  (was `Mod+W`).
- `Mod+H` and `Mod+L` are *also* confirmed host-captured, but window-focus
  movement is bound three redundant ways in `bindings.lua`'s `focusKeys`
  table — arrow keys (`Mod+Left/Right/Up/Down`, the one scheme confirmed
  safe here), `Mod+U/I/P` (left/down/right), and `Mod+H/J/K/L`
  (left/down/up/right, kept despite `H`/`L` not working on this host,
  for muscle memory and other deployments where nothing intercepts them).
  `Mod+O` was dropped from the `U/I/O/P` set entirely rather than kept as
  a dead bind, once it turned out captured too. Before adding any new
  global bind, check it against the confirmed-bad list (`Q`/`W`/`O`/`H`/`L`)
  first, and consider a redundant second binding if the action matters.
- The dwindle pseudo-tile toggle is `Mod+S` (moved twice: `Mod+P` → `Mod+K`
  when `P` was claimed by right-focus, then `Mod+K` → `Mod+S` once `K`
  was also claimed, by up-focus in the `H/J/K/L` scheme).
- `Mod+Y` opens the file manager (`$fm`, currently `cosmic-files`). It
  used to run yazi (`foot yazi`), dropped since there's no yazi in this
  setup anymore; the key was then reused for files. The scratchpad workflow (`Mod+S`/`Shift+S`) was also dropped —
  freeing `Mod+S` up for the pseudo-tile toggle above.
  `Mod+B` launches `$browser`, which now points at `brave-origin` (before
  that `chromium`, `firefox`, and `zen-browser`; none is in any package list
  anymore). Chromium replaced Firefox to get rid of GTK, and Brave Origin
  replaced Chromium, see "No Qt, no GTK".
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
starting size — see the `float-by-default` rule (`float = true, size = "960
540", center = true`) in `windows.lua`. This is why `tile.sh`/`float.sh`
exist: tiling isn't the default state, it's something you opt into
per-workspace via `Mod+T`.

## Night-time blue-light filter

`hyprsunset.conf` (sourced by nothing directly — read by the `hyprsunset`
binary itself, autostarted via `hl.exec_cmd("hyprsunset")` in
`autostart.lua`) defines two `profile` blocks: `identity = true` at
`07:00` and `temperature = 3000` at `20:00`. Don't drop the `identity`
profile — without it `hyprsunset` tints the screen by default even during
the day. Note `hyprsunset.conf`/`hypridle.conf` are their own tools with
their own config format — they weren't part of the Lua migration above and
have no `.lua` counterpart.

## Shell prompt

`hyprland/shell/zshrc` sets `PS1` to a macOS Terminal.app-style prompt
(`hostname:path %`, red `#` for root) via plain zsh `PROMPT_SUBST` — no
powerlevel10k or other framework. It's deployed to `~/.zshrc` by
`restore.sh`, `restore-safe.sh`, and `2-user.sh` (the x86_64
disk-partitioning installer path) alike, alongside `zshenv`/`zprofile` —
all three paths share the same shell setup now.

## Default terminal

`foot` is the only terminal — `terminal = "foot"` in `programs.lua`. Rio,
kitty, and alacritty were removed (config, keybinds, and package-list
entries) since only foot was ever actually installed/used; rio in particular
was referenced in configs (`$terminal`, windowrules) but never installed
anywhere. Don't reintroduce another terminal without removing the old one
everywhere it's referenced: `programs.lua`, `input.lua` (the
`foot-scroll-speed` window rule targets `match = { class = "foot" }`), and
the package lists in `1-setup.sh`, `hyprland/restore.sh`,
`hyprland/restore-safe.sh`.

Background transparency is set natively in `dotfiles/foot/foot.ini` via
`[colors-dark]` → `alpha=0.92` (not a compositor `windowrule = ... opacity`
— that dims the text too, not just the background). Note: this foot version
deprecated the old `[colors]` section name in favor of `[colors-dark]` /
`[colors-light]`; `foot --check-config` warns if you use the old name.

