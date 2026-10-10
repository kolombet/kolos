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
    foot
    fuzzel
    cosmic-files
    # File-chooser portal without GTK/Qt, for Brave Origin (see dotfiles/xdg-desktop-portal)
    xdg-desktop-portal
    xdg-desktop-portal-cosmic
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

echo "==> nogtk3 placeholder, then Brave Origin (see CLAUDE.md \"No Qt, no GTK\")"
# brave-origin-bin hard-depends on the gtk3 name; nogtk3 provides it so GTK3
# itself never gets installed (and gets replaced if it already is).
if [[ "$(id -u)" != "0" ]]; then
    NOGTK3_BUILD=$(mktemp -d)
    cp "$SCRIPT_DIR/nogtk3/PKGBUILD" "$NOGTK3_BUILD/"
    (cd "$NOGTK3_BUILD" && makepkg --noconfirm)
    # --ask 4: answer yes to "remove conflicting gtk3?"
    $PRIV pacman -U --noconfirm --needed --ask 4 "$NOGTK3_BUILD"/nogtk3-*.pkg.tar.*

    # AUR-only (packaged by Brave itself). Built with plain makepkg rather than
    # yay/paru so it doesn't depend on an AUR helper. Its package declares only
    # alsa-lib gtk3 libxss nss ttf-font (ttf-font: noto-fonts), but the binary
    # also links libraries the real gtk3 would have pulled in, which nogtk3
    # doesn't; install those explicitly so nothing treats them as orphans.
    $PRIV pacman -S --noconfirm --needed alsa-lib libxss nss \
        at-spi2-core cairo pango libcups libxcomposite libxdamage libxrandr \
        libxkbcommon mesa
    BRAVE_BUILD=$(mktemp -d)
    git clone --depth 1 https://aur.archlinux.org/brave-origin-bin.git "$BRAVE_BUILD"
    (cd "$BRAVE_BUILD" && makepkg --noconfirm)
    $PRIV pacman -U --noconfirm --needed "$BRAVE_BUILD"/brave-origin-bin-*.pkg.tar.*
else
    echo "  WARNING: makepkg can't run as root; skipping nogtk3 and Brave Origin (no browser installed)."
fi

echo "==> Installing kickoff from AUR"
yay -S --noconfirm --needed kickoff

echo "==> Backing up existing configs to $BACKUP_DIR"
mkdir -p "$BACKUP_DIR"
[[ -f "$USER_HOME/.zshenv" ]]         && cp    "$USER_HOME/.zshenv"          "$BACKUP_DIR/"
[[ -f "$USER_HOME/.zshrc" ]]          && cp    "$USER_HOME/.zshrc"           "$BACKUP_DIR/"
[[ -f "$USER_HOME/.ssh/config" ]]     && cp    "$USER_HOME/.ssh/config"      "$BACKUP_DIR/ssh_config"
[[ -d "$USER_HOME/.config/foot" ]]   && cp -r "$USER_HOME/.config/foot"     "$BACKUP_DIR/"

echo "==> Deploying dotfiles"
mkdir -p "$USER_HOME/.config"
cp -r "$SCRIPT_DIR/dotfiles/hypr"    "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/kickoff" "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/dotfiles/xdg-desktop-portal" "$USER_HOME/.config/"
cp -r "$SCRIPT_DIR/../dotfiles/foot" "$USER_HOME/.config/"

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

echo "==> Pacman: enabling parallel downloads"
if ! grep -q "^ParallelDownloads" /etc/pacman.conf; then
    $PRIV sed -i 's/^#ParallelDownloads/ParallelDownloads/' /etc/pacman.conf
fi

echo "==> Checking for Qt/GTK (kolos has neither, see CLAUDE.md \"No Qt, no GTK\")"
TOOLKIT_PKGS=$(pacman -Qq 2>/dev/null | grep -xE 'qt[56]-base|gtk[234]' || true)
if [[ -n "$TOOLKIT_PKGS" ]]; then
    echo "  WARNING: installed anyway: $(echo $TOOLKIT_PKGS). Required by:"
    pacman -Qi $TOOLKIT_PKGS | sed -n 's/^Required By *: /    /p'
fi

echo ""
echo "==> Done! Backup of overwritten files is at: $BACKUP_DIR"
echo ""
echo "Next steps:"
echo "  1. ~/.config/hypr/monitors.lua ships with a generic preferred/auto default;"
echo "     edit it in place if this machine needs a specific mode/position/scale."
echo "  2. Log out and start Hyprland, or run: Hyprland"
