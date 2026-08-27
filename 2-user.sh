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
'kickoff'
'nwg-wrapper'
)

yay -Syu --noconfirm --needed "${PKGS[@]}"

export PATH=$PATH:~/.local/bin
cp -r $HOME/$SCRIPTHOME/dotfiles/* $HOME/.config/
cp -r $HOME/$SCRIPTHOME/hyprland/dotfiles/* $HOME/.config/
cp $HOME/$SCRIPTHOME/hyprland/shell/zshenv $HOME/.zshenv
cp $HOME/$SCRIPTHOME/hyprland/shell/zprofile $HOME/.zprofile
cp $HOME/$SCRIPTHOME/hyprland/shell/zshrc $HOME/.zshrc

echo -e "\nDone!\n"
exit
