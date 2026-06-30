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
    hyprpaper
    waybar
    # Terminals
    foot
    alacritty
    kitty

    # Launchers / file manager
    fuzzel
    pcmanfm

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
)

$PRIV pacman -Syu --noconfirm --needed "${PKGS[@]}"

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
cp -r "$SCRIPT_DIR/dotfiles/waybar"  "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/kickoff" "$USER_HOME/.config/"

echo "==> Shell configs"
cp "$SCRIPT_DIR/shell/zshenv"   "$USER_HOME/.zshenv"
cp "$SCRIPT_DIR/shell/zprofile" "$USER_HOME/.zprofile"

echo "==> Setting zsh as default shell for $(whoami)"
$PRIV chsh -s /usr/bin/zsh "$(whoami)"

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

echo ""
echo "==> Done!"
echo ""
echo "Next steps:"
echo "  1. Review ~/.config/hypr/hyprland.conf — update the monitor= line for your display."
echo "     For a real laptop: monitor=,preferred,auto,1"
echo "     For a UTM/QEMU VM: monitor=,3024x1964@60,0x0,2  (adjust to host resolution)"
echo "  2. Reboot or log out and log back in to start Hyprland via ~/.zprofile."
