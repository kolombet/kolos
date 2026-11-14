#!/usr/bin/env bash
#-------------------------------------------------------------------------
#  ██╗  ██╗ ██████╗ ██╗      ██████╗ ███████╗
#  ██║ ██╔╝██╔═══██╗██║     ██╔═══██╗██╔════╝
#  █████╔╝ ██║   ██║██║     ██║   ██║███████╗
#  ██╔═██╗ ██║   ██║██║     ██║   ██║╚════██║
#  ██║  ██╗╚██████╔╝███████╗╚██████╔╝███████║
#  ╚═╝  ╚═╝ ╚═════╝ ╚══════╝ ╚═════╝ ╚══════╝
#-------------------------------------------------------------------------

echo -e "\nINSTALLING AUR SOFTWARE\n"
# You can solve users running this script as root with this and then doing the same for the next for statement. However I will leave this up to you.

echo "CLONING: YAY"
cd ~
git clone "https://aur.archlinux.org/yay.git"
cd ${HOME}/yay
makepkg -si --noconfirm
cd ~

PKGS=(
'rust'
'brave-origin-bin'
)

yay -Syu --noconfirm --needed "${PKGS[@]}"

export PATH=$PATH:~/.local/bin
# kolos-launcher (Mod+D) is built from this repo, not installed from the AUR.
cargo build --release --manifest-path $HOME/$SCRIPTHOME/launcher/Cargo.toml
install -Dm755 $HOME/$SCRIPTHOME/launcher/target/release/kolos-launcher $HOME/.local/bin/kolos-launcher
cp -r $HOME/$SCRIPTHOME/dotfiles/* $HOME/.config/
cp -r $HOME/$SCRIPTHOME/hyprland/dotfiles/* $HOME/.config/
mkdir -p $HOME/.local/share/konsole
cp $HOME/$SCRIPTHOME/dotfiles/konsole/Foot.colorscheme $HOME/$SCRIPTHOME/dotfiles/konsole/Foot.profile $HOME/.local/share/konsole/
cp $HOME/$SCRIPTHOME/dotfiles/konsole/konsolerc $HOME/.config/konsolerc
cp $HOME/$SCRIPTHOME/hyprland/shell/zshenv $HOME/.zshenv
cp $HOME/$SCRIPTHOME/hyprland/shell/zprofile $HOME/.zprofile
cp $HOME/$SCRIPTHOME/hyprland/shell/zshrc $HOME/.zshrc

echo -e "\nDone!\n"
exit
