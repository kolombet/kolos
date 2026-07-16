#!/bin/bash
# Print the working directory of the most-recently-focused foot window's
# shell, so a newly opened terminal starts in the same directory (macOS
# Terminal.app behavior). Falls back to $HOME.

pid=$(hyprctl clients -j 2>/dev/null | jq -r '
    [.[] | select(.class == "foot")] | sort_by(.focusHistoryID) | .[0].pid // empty
')

if [ -z "$pid" ] || [ "$pid" = "null" ]; then
    echo "$HOME"
    exit 0
fi

# foot's direct child is the login shell. Deliberately not walking further
# down into whatever the shell is running — a foreground process (like an
# editor) that forked from the shell shares the same cwd anyway, and
# anything that spawns its own ephemeral subprocess tree (e.g. a nested
# Claude Code session) would make a deeper walk noisy and unreliable.
shell_pid=$(pgrep -P "$pid" | head -n1)

if [ -n "$shell_pid" ] && [ -r "/proc/$shell_pid/cwd" ]; then
    readlink -f "/proc/$shell_pid/cwd"
else
    echo "$HOME"
fi
