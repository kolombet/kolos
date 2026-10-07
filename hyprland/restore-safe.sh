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
    fuzzel
    pcmanfm
    firefox
    pipewire
    pipewire-audio
    pipewire-pulse
    wireplumber
    noto-fonts
    ttf-jetbrains-mono-nerd
    zsh
    git
    base-devel

    # Build toolchain for askpass/ (kolos-askpass GUI sudo password prompt)
    rust
)
$PRIV pacman -Syu --noconfirm --needed "${PKGS[@]}"

echo "==> Installing kickoff and nwg-wrapper from AUR"
yay -S --noconfirm --needed kickoff nwg-wrapper

echo "==> Backing up existing configs to $BACKUP_DIR"
mkdir -p "$BACKUP_DIR"
[[ -d "$USER_HOME/.config/waybar" ]]  && cp -r "$USER_HOME/.config/waybar"  "$BACKUP_DIR/"
[[ -f "$USER_HOME/.zshenv" ]]         && cp    "$USER_HOME/.zshenv"          "$BACKUP_DIR/"
[[ -f "$USER_HOME/.zshrc" ]]          && cp    "$USER_HOME/.zshrc"           "$BACKUP_DIR/"
[[ -f "$USER_HOME/.ssh/config" ]]     && cp    "$USER_HOME/.ssh/config"      "$BACKUP_DIR/ssh_config"

echo "==> Deploying dotfiles"
mkdir -p "$USER_HOME/.config"
cp -r "$SCRIPT_DIR/dotfiles/hypr"    "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/waybar"  "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/kickoff" "$USER_HOME/.config/"

echo "==> Shell configs"
cp "$SCRIPT_DIR/shell/zshenv"   "$USER_HOME/.zshenv"
cp "$SCRIPT_DIR/shell/zprofile" "$USER_HOME/.zprofile"
cp "$SCRIPT_DIR/shell/zshrc"    "$USER_HOME/.zshrc"

echo "==> Setting zsh as default shell"
$PRIV chsh -s /usr/bin/zsh "$(whoami)"

echo "==> Building askpass/ (kolos-askpass GUI sudo password prompt)"
cargo build --release --manifest-path "$SCRIPT_DIR/../askpass/Cargo.toml"
$PRIV install -Dm755 "$SCRIPT_DIR/../askpass/target/release/kolos-askpass" /usr/local/bin/kolos-askpass
if grep -q "^Path askpass" /etc/sudo.conf 2>/dev/null; then
    $PRIV sed -i 's|^Path askpass .*|Path askpass /usr/local/bin/kolos-askpass|' /etc/sudo.conf
else
    echo "Path askpass /usr/local/bin/kolos-askpass" | $PRIV tee -a /etc/sudo.conf >/dev/null
fi

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

echo "==> Pacman: enabling parallel downloads"
if ! grep -q "^ParallelDownloads" /etc/pacman.conf; then
    $PRIV sed -i 's/^#ParallelDownloads/ParallelDownloads/' /etc/pacman.conf
fi

echo "==> Checking for Qt (kolos is Qt-free, see CLAUDE.md \"No Qt\")"
QT_PKGS=$(pacman -Qq 2>/dev/null | grep -xE 'qt[56]-base' || true)
if [[ -n "$QT_PKGS" ]]; then
    echo "  WARNING: Qt is installed ($(echo $QT_PKGS)). Required by:"
    pacman -Qi $QT_PKGS | sed -n 's/^Required By *: /    /p'
fi

echo ""
echo "==> Done! Backup of overwritten files is at: $BACKUP_DIR"
echo ""
echo "Next steps:"
echo "  1. ~/.config/hypr/monitors.lua ships with a generic preferred/auto default;"
echo "     edit it in place if this machine needs a specific mode/position/scale."
echo "  2. Log out and start Hyprland, or run: Hyprland"
