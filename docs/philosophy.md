# Philosophy

Why this exists, distinct from `CLAUDE.md` (the "how") and
`docs/user-stories.md` (the "why" behind individual config decisions). This
file is the "what for" — the thesis the rest of the repo is built to serve.

kolos is not trying to be a friendly, batteries-included desktop distro.
It's a small set of primitives — a compositor, a terminal, a browser, a
launcher, a status bar — wired together with as little in between as
possible, on the bet that a system this small is easier to understand, fix,
and reshape than one that ships a feature for every possible need up front.

## Minimal

No bells and whistles. One terminal (`foot`), one launcher (`kickoff`), one
status bar (`waybar`), one browser. Every app, package, or config knob has
to earn its place — see `CLAUDE.md`'s "Default terminal" section for what
"only one of anything" looks like in practice (rio/kitty/alacritty removed
once foot was the only one actually used). Fewer moving parts means fewer
things that can break, and less to read before you understand the whole
system.

## Modern

Wayland and Hyprland, not X11 and a stacking WM from 2005. Compositing,
scaling, and multi-monitor handling are the compositor's job natively, not
bolted on with `xrandr` and a compositor daemon fighting each other.

## Fast

Rendering is GPU-accelerated end to end, the way a game engine renders a
scene, not a stack of software-composited widget toolkits. Config is Lua,
not a bespoke parser — same reasoning game engines script logic in Lua
over building a custom DSL: fast to load, fast to evaluate, and a real
language instead of an ad hoc config grammar. See "Hyprland config layout"
in `CLAUDE.md` for how this repo migrated onto that as of Hyprland 0.55+.

## AI-optimized

The system is meant to be maintained by an agent as much as by hand. That's
why `CLAUDE.md` exists and is kept this detailed, why config changes in
this repo get verified against the installed type stub and a live
`hyprctl repl` session instead of guessed from memory or the wiki (see the
Lua migration notes), and why breakage is treated as cheap to fix rather
than something to engineer around defensively. A minimal, legible system is
also the system an agent can most reliably reason about and repair —
minimalism and AI-maintainability reinforce each other here, they're not
separate goals.

## Flexible

Pieces are meant to interlock like puzzle pieces, not fuse together.
Swapping the terminal, the launcher, the browser, or the status bar should
mean editing one config file and one package-list entry, not untangling
assumptions baked in across a dozen files. `programs.lua`'s
`terminal`/`browser`/`menu`/`apps` globals exist specifically so the rest
of the config never hardcodes an app name directly.

## Stable

Stable as in settled, not trend-chasing — the interaction model and visual
language don't need to be reinvented every year. The look deliberately
evokes Windows 7's Aero/glass era (`looknfeel.lua`'s blur, rounded corners,
and translucency) over whatever the current desktop-Linux aesthetic trend
is: familiar, comfortable, and not something that needs to be redesigned
next time you sit down at it.

## Keyboard-oriented

Everything reachable without a mouse: launching apps, moving and resizing
windows, switching workspaces, tiling/floating a whole workspace at once.
`bindings.lua` is the source of truth. The one deliberate exception is
window drag/resize by holding `Mod` and dragging with the mouse — offered
as a convenience, never required.

## Portable

Runs anywhere Linux runs — x86_64 and aarch64 alike (see `1-setup.sh`'s ARM
Wi-Fi note and `README.md`'s "Testing in a Virtual Machine" section for the
Apple Silicon/UTM path specifically). No distro-specific packages or paths;
see "Portable, self-contained restore" in `docs/user-stories.md`.

## Grow on demand

The base system ships with the minimum to be usable, not everything you
might eventually want. Need a feature? Add it. Need a program that doesn't
exist yet? Generate it. The system is meant to be shaped incrementally, by
you or an agent, into whatever a specific machine or workflow actually
needs — not pre-loaded with every tool on the chance it gets used someday.

## VM-first, bare-metal-ready

Developed and iterated on primarily inside a VM (`docs/emulator-testing.md`)
because that loop is fast and disposable, but every machine-specific
difference between a VM guest and real hardware is called out explicitly
rather than papered over — see `monitors.lua` (per-machine, hand-edited)
and "Suspend-on-idle is laptop-only" in `CLAUDE.md`. The goal is a system
that's genuinely fine on both, not one that only works in the environment
it was built in.

## Layout follows the screen

The intent: tiled by default on small laptop screens, where every pixel of
horizontal space matters, and floating by default on desktop-sized
monitors, where it doesn't. **Not yet how the config actually behaves** —
`windows.lua`'s `float-by-default` rule currently applies everywhere
regardless of screen size or machine type; `Mod+T`/`Mod+F` exist to toggle
a whole workspace between the two manually in the meantime. Making the
default itself screen-size-aware is open work, not a solved acceptance
criterion — don't describe it as already done.
