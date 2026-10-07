#!/usr/bin/env bash
# Hyprland restore script
# Works on Arch Linux (x86_64 and aarch64).
# Run from a minimal Arch install (after network is up).
# Can be run as root or as a regular user with sudo.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
USER_HOME="${HOME}"
ARCH="$(uname -m)"

# privilege helper — no-op when already root
if [[ "$(id -u)" == "0" ]]; then
    PRIV=""
else
    PRIV="sudo"
    echo "Running as non-root; will use sudo for system commands."
fi

echo "==> Architecture: $ARCH"
echo "==> Installing packages"

PKGS=(
    # Wayland / Hyprland
    hyprland
    swaybg
    # Terminal
    foot

    # Launchers / file manager
    fuzzel
    cosmic-files
    # File-chooser portal without GTK/Qt, for Chromium (see dotfiles/xdg-desktop-portal)
    xdg-desktop-portal
    xdg-desktop-portal-cosmic

    # Audio (PipeWire stack)
    pipewire
    pipewire-audio
    pipewire-pulse
    wireplumber

    # Fonts
    noto-fonts
    ttf-jetbrains-mono-nerd

    # Seat management (required for Hyprland as root or non-root)
    seatd

    # Shell and essentials
    zsh
    git
    base-devel

    # Build toolchain for askpass/ (kolos-askpass GUI sudo password prompt)
    rust
)

$PRIV pacman -Syu --noconfirm --needed "${PKGS[@]}"

echo "==> nogtk3 placeholder, then Chromium (see CLAUDE.md \"No Qt, no GTK\")"
# chromium's package hard-depends on the gtk3 name; nogtk3 provides it so GTK3
# itself never gets installed (and gets replaced if it already is).
if [[ "$(id -u)" != "0" ]]; then
    NOGTK3_BUILD=$(mktemp -d)
    cp "$SCRIPT_DIR/nogtk3/PKGBUILD" "$NOGTK3_BUILD/"
    (cd "$NOGTK3_BUILD" && makepkg --noconfirm)
    # --ask 4: answer yes to "remove conflicting gtk3?"
    $PRIV pacman -U --noconfirm --needed --ask 4 "$NOGTK3_BUILD"/nogtk3-*.pkg.tar.*
else
    echo "  WARNING: makepkg can't run as root; skipping nogtk3, so chromium will pull in gtk3."
fi
$PRIV pacman -S --noconfirm --needed chromium

echo "==> Enabling seatd"
$PRIV systemctl enable --now seatd.service

# AUR helper check (kickoff is AUR-only)
AUR_PKGS=(kickoff)
if command -v yay &>/dev/null; then
    yay -S --noconfirm --needed "${AUR_PKGS[@]}"
elif command -v paru &>/dev/null; then
    paru -S --noconfirm --needed "${AUR_PKGS[@]}"
else
    echo "Warning: no AUR helper (yay/paru) found."
    echo "Install kickoff manually: https://github.com/j0ru/kickoff"
fi

echo "==> Deploying dotfiles to $USER_HOME/.config"
mkdir -p "$USER_HOME/.config"

cp -r "$SCRIPT_DIR/dotfiles/hypr"    "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/kickoff" "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/xdg-desktop-portal" "$USER_HOME/.config/"

echo "==> Shell configs"
cp "$SCRIPT_DIR/shell/zshenv"   "$USER_HOME/.zshenv"
cp "$SCRIPT_DIR/shell/zprofile" "$USER_HOME/.zprofile"
cp "$SCRIPT_DIR/shell/zshrc"    "$USER_HOME/.zshrc"

echo "==> Setting zsh as default shell for $(whoami)"
$PRIV chsh -s /usr/bin/zsh "$(whoami)"

echo "==> Building askpass/ (kolos-askpass GUI sudo password prompt)"
cargo build --release --manifest-path "$SCRIPT_DIR/../askpass/Cargo.toml"
$PRIV install -Dm755 "$SCRIPT_DIR/../askpass/target/release/kolos-askpass" /usr/local/bin/kolos-askpass
if grep -q "^Path askpass" /etc/sudo.conf 2>/dev/null; then
    $PRIV sed -i 's|^Path askpass .*|Path askpass /usr/local/bin/kolos-askpass|' /etc/sudo.conf
else
    echo "Path askpass /usr/local/bin/kolos-askpass" | $PRIV tee -a /etc/sudo.conf >/dev/null
fi

echo "==> Building keys/ (kolos-keys keybinding cheat sheet, toggled with Mod+\`)"
cargo build --release --manifest-path "$SCRIPT_DIR/../keys/Cargo.toml"
install -Dm755 "$SCRIPT_DIR/../keys/target/release/kolos-keys" "$USER_HOME/.local/bin/kolos-keys"

echo "==> Building bar/ (kolos-bar, the status bar)"
cargo build --release --manifest-path "$SCRIPT_DIR/../bar/Cargo.toml"
install -Dm755 "$SCRIPT_DIR/../bar/target/release/kolos-bar" "$USER_HOME/.local/bin/kolos-bar"

echo "==> SSH agent (OpenSSH ssh-agent + kolos-askpass instead of gcr-ssh-agent)"
if [[ "$(id -u)" != "0" ]]; then
    systemctl --user disable --now gcr-ssh-agent.socket gcr-ssh-agent.service 2>/dev/null || true
    systemctl --user enable --now ssh-agent.socket
else
    echo "  Skipped agent swap (user units; rerun as the desktop user)."
fi
mkdir -p "$USER_HOME/.ssh" && chmod 700 "$USER_HOME/.ssh"
touch "$USER_HOME/.ssh/config" && chmod 600 "$USER_HOME/.ssh/config"
if ! grep -q "^AddKeysToAgent" "$USER_HOME/.ssh/config"; then
    { echo "AddKeysToAgent yes"; echo; cat "$USER_HOME/.ssh/config"; } > "$USER_HOME/.ssh/config.new"
    mv "$USER_HOME/.ssh/config.new" "$USER_HOME/.ssh/config"
fi

echo "==> Pacman tweaks"
# Enable parallel downloads if not already set
if ! grep -q "^ParallelDownloads" /etc/pacman.conf; then
    $PRIV sed -i 's/^#ParallelDownloads/ParallelDownloads/' /etc/pacman.conf
fi
# On kernels without Landlock LSM (e.g. some ARM/QEMU kernels), the
# DownloadUser sandbox fails. Comment it out if pacman errors with:
# "Landlock is not supported by the kernel"
if grep -q "^DownloadUser" /etc/pacman.conf; then
    echo "  NOTE: If you see 'Landlock is not supported' errors from pacman,"
    echo "  comment out 'DownloadUser' in /etc/pacman.conf"
fi

echo "==> Autologin on tty1"
if [[ "$(id -u)" == "0" ]]; then
    read -rp "  Set up root autologin on tty1? [y/N] " ans
    if [[ "${ans,,}" == "y" ]]; then
        $PRIV mkdir -p /etc/systemd/system/getty@tty1.service.d
        $PRIV cp "$SCRIPT_DIR/system/getty-autologin.conf" \
                 /etc/systemd/system/getty@tty1.service.d/override.conf
        $PRIV systemctl daemon-reload
        echo "  -> Autologin enabled."
    fi
else
    echo "  Skipped (autologin drop-in is root-only). To enable manually:"
    echo "    sudo mkdir -p /etc/systemd/system/getty@tty1.service.d"
    echo "    sudo cp $SCRIPT_DIR/system/getty-autologin.conf \\"
    echo "         /etc/systemd/system/getty@tty1.service.d/override.conf"
    echo "    sudo systemctl daemon-reload"
fi

echo "==> Checking for Qt/GTK (kolos has neither, see CLAUDE.md \"No Qt, no GTK\")"
TOOLKIT_PKGS=$(pacman -Qq 2>/dev/null | grep -xE 'qt[56]-base|gtk[234]' || true)
if [[ -n "$TOOLKIT_PKGS" ]]; then
    echo "  WARNING: installed anyway: $(echo $TOOLKIT_PKGS). Required by:"
    pacman -Qi $TOOLKIT_PKGS | sed -n 's/^Required By *: /    /p'
fi

echo ""
echo "==> Done!"
echo ""
echo "Next steps:"
echo "  1. ~/.config/hypr/monitors.lua ships with a generic preferred/auto default;"
echo "     edit it in place if this machine needs a specific mode/position/scale"
echo "     (e.g. a UTM/QEMU VM's virtual output, or a multi-monitor layout)."
echo "  2. Reboot or log out and log back in to start Hyprland via ~/.zprofile."
