# Shared by restore.sh and restore-safe.sh (sourced; expects SCRIPT_DIR, USER_HOME and PRIV).
# Installs from the repos where a package exists and builds what doesn't, so one script serves
# Arch/Manjaro (everything in the repos) and Pocknix's locked ARM64 snapshot (much of it missing).

# Pocknix (Arch Linux ARM handheld distro) owns tty1, seatd-free seating, gtk3 and its updates.
is_pocknix() { grep -qx 'NAME="Pocknix OS"' /etc/os-release 2>/dev/null; }

in_repos()  { pacman -Si "$1" &>/dev/null; }
installed() { pacman -Q "$1" &>/dev/null; }

# Order matters: each needs the ones before it (foot <- fcft <- libutf8proc, tllist).
BUILDABLE=(tllist libutf8proc fcft foot swaybg zsh)

build_pkg() {   # <name>: pkgbuilds/<name>/PKGBUILD with makepkg, installed through pacman
    local dir
    dir=$(mktemp -d)
    cp "$SCRIPT_DIR/pkgbuilds/$1/"* "$dir/"
    (cd "$dir" && makepkg -si --noconfirm --needed)
    rm -rf "$dir"
}

# <required pkgs...> -- <optional pkgs...>: optional ones are skipped when no repo has them.
install_packages() {
    local p repo=() optional=0 skipped=()
    for p in "$@"; do
        [[ "$p" == -- ]] && { optional=1; continue; }
        if in_repos "$p"; then repo+=("$p")
        elif installed "$p"; then :
        elif [[ " ${BUILDABLE[*]} " == *" $p "* || "$p" == ttf-jetbrains-mono-nerd || "$p" == rust ]]; then :
        else skipped+=("$p$( ((optional)) || echo ' (REQUIRED)')"); fi
    done
    # Pocknix updates through its snapshot-protected flow; a bare -Syu would bypass it.
    if is_pocknix; then
        ((${#repo[@]})) && $PRIV pacman -S --noconfirm --needed "${repo[@]}"
    else
        $PRIV pacman -Syu --noconfirm --needed "${repo[@]}"
    fi

    if [[ "$(id -u)" == "0" ]]; then
        echo "  WARNING: makepkg can't run as root; not building: $(missing_buildable)"
    else
        for p in "${BUILDABLE[@]}"; do
            installed "$p" || in_repos "$p" || { echo "==> Building $p (not in the repos)"; build_pkg "$p"; }
        done
    fi
    ((${#skipped[@]})) && echo "  WARNING: in no repo, skipped: ${skipped[*]}"
    install_nerd_font
    ensure_cargo
}

missing_buildable() {
    local p out=()
    for p in "${BUILDABLE[@]}"; do installed "$p" || in_repos "$p" || out+=("$p"); done
    echo "${out[*]:-none}"
}

# ttf-jetbrains-mono-nerd's PKGBUILD builds every Nerd Font; fetch just this one instead.
install_nerd_font() {
    installed ttf-jetbrains-mono-nerd && return 0
    fc-list | grep -q "JetBrainsMono Nerd Font" && return 0
    echo "==> JetBrains Mono Nerd Font (not in the repos): from the Nerd Fonts release"
    local dir="$USER_HOME/.local/share/fonts/JetBrainsMonoNerd"
    mkdir -p "$dir"
    curl -fsSL https://github.com/ryanoasis/nerd-fonts/releases/download/v3.4.0/JetBrainsMono.tar.xz \
        | tar -xJ -C "$dir"
    fc-cache -f "$dir" >/dev/null
}

# No rust package (Pocknix): the official rustup toolchain in ~/.cargo, for this user only.
ensure_cargo() {
    command -v cargo &>/dev/null && return 0
    [[ -x "$USER_HOME/.cargo/bin/cargo" ]] || {
        echo "==> Rust (not in the repos): rustup into ~/.cargo"
        curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --no-modify-path
    }
    export PATH="$USER_HOME/.cargo/bin:$PATH"
}

# kickoff from crates.io instead of the AUR: no yay/paru needed, same on every distro.
install_tools() {
    echo "==> kickoff (launcher) from crates.io"
    cargo install --locked --root "$USER_HOME/.local" kickoff

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
}

# monitors.lua is hand-edited per machine (CLAUDE.md); a redeploy must not reset it.
deploy_hypr() {
    local keep=""
    [[ -f "$USER_HOME/.config/hypr/monitors.lua" ]] && keep=$(mktemp) \
        && cp "$USER_HOME/.config/hypr/monitors.lua" "$keep"
    cp -r "$SCRIPT_DIR/dotfiles/hypr" "$USER_HOME/.config/"
    [[ -n "$keep" ]] && mv "$keep" "$USER_HOME/.config/hypr/monitors.lua"
    return 0
}
