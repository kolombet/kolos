# kolos

Personal Arch Linux system configuration. Two independent pieces live here:

1. **Automated installer** (repo root) — partitions a disk, installs Arch + Hyprland.
2. **Hyprland dotfiles** (`hyprland/`) — a Wayland setup (Hyprland + waybar) that is installed automatically or can be laid on top of an existing Arch install.

Config files are plain copies deployed into `~/.config` — there's no symlink manager. Changes are made in this repo and then copied over by hand (or via the `restore*.sh` scripts for a first-time deploy).

---

## Repo layout

```
0-preinstall.sh       # disk partitioning (btrfs), base pacstrap, systemd-boot
1-setup.sh            # base packages, locale/mirrors
2-user.sh             # yay, deploys shell config + dotfiles/
3-post-setup.sh       # enables services (cups, ntpd, bluetooth, NetworkManager)
install.sh            # orchestrates the three scripts above via arch-chroot
install.example.conf  # hostname/username/password template consumed mid-install
setconsole.sh         # sets TTY keymap/font (KEYMAP=us, FONT=ter-v16b)

dotfiles/              # editor/terminal configs, deployed to ~/.config by 2-user.sh
  foot/                # foot terminal (the only terminal — theme include + keybindings)
  helix/                # Helix editor config
  nvim/                 # LazyVim-based Neovim config
  zed/                  # Zed editor settings

hyprland/               # standalone Hyprland desktop setup (see below)
  restore.sh            # fresh-install path: installs packages, deploys dotfiles, sets up zsh/autologin
  restore-safe.sh       # same, but backs up any existing ~/.config first instead of overwriting
  dotfiles/hypr/        # hyprland.conf, window-mode scripts, wallpapers
  dotfiles/waybar/      # waybar config + style.css
  dotfiles/kickoff/     # kickoff (app launcher) config
  shell/                # zprofile (autostarts Hyprland on tty1), zshenv, zshrc (prompt)
  system/               # getty-autologin.conf drop-in for tty1 autologin
```

---

## Path 1: Automated Arch + Hyprland install

Boots a bare Arch ISO all the way to a themed Hyprland desktop.

Download an Arch ISO from <https://archlinux.org/download/> and write it to a USB drive (Ventoy, Etcher, etc.). Boot it, then:

```bash
pacman -Sy git
git clone https://github.com/kolombet/kolos
cd kolos
./install.sh
```

This runs, in order: `0-preinstall.sh` (partition + pacstrap + bootloader) → `1-setup.sh` (packages) → `2-user.sh` (AUR helper, zsh, dotfiles) → `3-post-setup.sh` (enable services, sudoers cleanup).

**No Wi-Fi during install?** The Arch ISO's live environment uses `iwd` — connect with `iwctl` before running `install.sh` (no internet to look this up otherwise, so it's spelled out here):

```
iwctl
[iwd]# device list
[iwd]# station <device> scan
[iwd]# station <device> get-networks
[iwd]# station <device> connect "<SSID>"
Passphrase: ...
[iwd]# exit
```

Once installed, the system itself manages Wi-Fi via **NetworkManager** (installed by `1-setup.sh`, enabled by `3-post-setup.sh`) — use `nmtui` or `nmcli` post-install. (An `nm-applet` autostart line exists in `hyprland/dotfiles/hypr/autostart.conf` but is commented out by default.)

### Testing in a Virtual Machine

Because the automated installer partitions and formats the disk, the safest way to test it without altering your host machine is inside a Virtual Machine.

1. **Get the Software:**
   - **macOS:** Download **[UTM](https://mac.getutm.app/)** (free) or VMware Fusion. *(Note for Apple Silicon users: You must emulate an x86_64 architecture in UTM to test this exact configuration, as it uses standard x86_64 Arch repos and microcode).*
   - **Windows/Linux:** Use VirtualBox or VMware Workstation.
   - Download the official [Arch Linux x86_64 ISO](https://archlinux.org/download/).
2. **Configure the VM:** 
   - Assign at least **4GB of RAM** and **2 CPU cores**.
   - Create a blank virtual drive (e.g., **30GB+**).
   - Ensure **UEFI boot** is enabled (required for `bootctl`).
3. **Boot & Run:** 
   - Mount the Arch ISO and boot the VM.
   - Follow the `pacman -Sy git` and `git clone` steps from above. 
   - When the script prompts for a disk to format, type the name of your VM's virtual drive (e.g., `/dev/vda` for UTM/QEMU or `/dev/sda` for VirtualBox). You can look at the output of the `lsblk` command that the script prints to verify the exact name.
## Path 2: Hyprland desktop (this machine)

A minimal Wayland setup: Hyprland + waybar + swaybg, laid on top of an already-installed Arch system.

```bash
cd hyprland
./restore.sh        # fresh machine — overwrites ~/.config/{hypr,waybar,kickoff}
# or
./restore-safe.sh   # backs up any existing configs to ~/.config-backup-<timestamp> first
```

Either script installs the required packages (`hyprland`, `waybar`, `swaybg`, `foot`, `kickoff`, PipeWire audio stack, fonts, `seatd`), copies `dotfiles/{hypr,waybar,kickoff}` into `~/.config`, installs the `zprofile`/`zshenv`/`zshrc` shell files, sets `zsh` as the login shell, and optionally wires up tty1 autologin. After that, logging into tty1 starts Hyprland automatically via `zprofile`. `zshrc` sets a macOS Terminal.app-style prompt (`hostname:path %`) — no external prompt framework (e.g. powerlevel10k) required.

**Notable behavior:**
- `Super+F` / `Super+T` run `float.sh` / `tile.sh` to float or tile all windows on the current workspace; `Super+V` toggles floating on the active window. New windows float by default (see the `windowrule` block in `hyprland.conf`).
- `Super+R` enters a resize submap (`H`/`J`/`K`/`L` to resize, `Return`/`Escape` to exit).
- On login, `randomwallpaper.sh` picks a random image from `hypr/wallpapers/` per monitor and sets it with `swaybg`.
- waybar shows CPU/temperature/memory/battery/clock and an EN/RU keyboard-layout indicator, styled with a Nerd Font.

Applying a config change from this repo to a running system is currently manual — edit the file under `hyprland/dotfiles/` or `dotfiles/`, copy it to the matching path under `~/.config`, then `hyprctl reload` (for Hyprland) or restart the relevant process (e.g. `killall waybar && waybar &`, or re-run `randomwallpaper.sh` for swaybg).

---

## Troubleshooting

__[Arch Linux Installation Guide](https://github.com/rickellis/Arch-Linux-Install-Guide)__

## Credits

- Original packages script was a post-install cleanup script called ArchMatic: <https://github.com/rickellis/ArchMatic>
- Base installer structure adapted from ArchTitus (<https://www.christitus.com/arch-titus>) and its livestream series: <https://www.youtube.com/watch?v=IkMCtkDIhe8&list=PLc7fktTRMBowNaBTsDHlL6X3P3ViX3tYg>

See [LICENSE](LICENSE) and [NOTICE](NOTICE) for this repo's license and the preserved upstream ArchTitus license text.
