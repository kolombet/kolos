#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
USER_HOME="${HOME}"
BACKUP_DIR="$USER_HOME/.config-backup-$(date +%Y%m%d_%H%M%S)"

PRIV=""
[[ "$(id -u)" != "0" ]] && PRIV="sudo"

echo "==> Installing packages"
PKGS=(
    hyprland
    swaybg
    waybar
    foot
    alacritty
    kitty
    fuzzel
    pcmanfm
    pipewire
    pipewire-audio
    pipewire-pulse
    wireplumber
    noto-fonts
    ttf-jetbrains-mono-nerd
    zsh
    git
    base-devel
)
$PRIV pacman -Syu --noconfirm --needed "${PKGS[@]}"

echo "==> Installing kickoff from AUR"
yay -S --noconfirm --needed kickoff

echo "==> Backing up existing configs to $BACKUP_DIR"
mkdir -p "$BACKUP_DIR"
[[ -d "$USER_HOME/.config/waybar" ]]  && cp -r "$USER_HOME/.config/waybar"  "$BACKUP_DIR/"
[[ -f "$USER_HOME/.zshenv" ]]         && cp    "$USER_HOME/.zshenv"          "$BACKUP_DIR/"

echo "==> Deploying dotfiles"
mkdir -p "$USER_HOME/.config"
cp -r "$SCRIPT_DIR/dotfiles/hypr"    "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/waybar"  "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/kickoff" "$USER_HOME/.config/"

echo "==> Shell configs"
cp "$SCRIPT_DIR/shell/zshenv"   "$USER_HOME/.zshenv"
cp "$SCRIPT_DIR/shell/zprofile" "$USER_HOME/.zprofile"

echo "==> Setting zsh as default shell"
$PRIV chsh -s /usr/bin/zsh "$(whoami)"

echo "==> Pacman: enabling parallel downloads"
if ! grep -q "^ParallelDownloads" /etc/pacman.conf; then
    $PRIV sed -i 's/^#ParallelDownloads/ParallelDownloads/' /etc/pacman.conf
fi

echo ""
echo "==> Done! Backup of overwritten files is at: $BACKUP_DIR"
echo ""
echo "Next steps:"
echo "  1. Review ~/.config/hypr/hyprland.conf — update the monitor= line for your display."
echo "     For a real display: monitor=,preferred,auto,1"
echo "  2. Log out and start Hyprland, or run: Hyprland"
