# Running the full install in a VM

How to test the automated installer (`install.sh` → `0-preinstall.sh` /
`1-setup.sh` / `2-user.sh` / `3-post-setup.sh`) end-to-end inside a VM,
instead of on real hardware. This is the fresh-install path — for laying the
Hyprland config on top of an *already-installed* Arch system, use
`hyprland/restore.sh` / `restore-safe.sh` instead (no VM-specific steps
needed there beyond what's below for GPU/display).

## VM setup

- **Architecture**: guest arch must match the Arch install ISO you boot
  (aarch64 guest → Arch Linux ARM ISO, x86_64 guest → Arch Linux ISO).
- **GPU/display device**: set the emulated display to a virtio-gpu variant
  with 3D/OpenGL acceleration (in UTM: `virtio-gpu-gl-pci`; in raw QEMU:
  `-device virtio-gpu-gl-pci -display <backend>,gl=on`). This gets Hyprland
  hardware-accelerated GLES2 rendering (`+virgl`) for free — no extra guest
  packages needed. Plain `virtio-gpu` (no `-gl`), `vga`, or `qxl` will *not*
  accelerate and Hyprland will fall back to software rendering (works, but
  slower).
- **Disk size**: no special sizing needed. `0-preinstall.sh` partitions
  1000M for `/boot` and gives the rest to a btrfs root — plenty of headroom
  for kernel/initramfs upgrades, unlike a hand-partitioned small `/boot`
  (200M+ can fill up over time on aarch64 unless you exclude
  `boot/dtbs/*`/`boot/Image.gz` from pacman extraction).
- **RAM/CPU**: no special requirements — Hyprland + waybar is light. A
  couple of cores and a few GB of RAM is enough to install and run
  comfortably.

## Installing

1. Boot the Arch install ISO in the VM.
2. Confirm networking is up: `ping -c1 archlinux.org`.
3. Clone this repo (or copy it in) and `cd` into it.
4. `cp install.example.conf install.conf` and edit `hostname`/`username`/
   `password`.
5. `bash install.sh` — this runs, in order: `0-preinstall.sh` (mirror setup,
   disk partition/format — **confirm it's pointed at the VM's virtual disk**,
   it wipes whatever you select), then `arch-chroot`s in to run
   `1-setup.sh` (base packages + Hyprland ecosystem), `2-user.sh` (yay, zsh,
   deploys `dotfiles/`), and `3-post-setup.sh` (services, autologin,
   `seatd`).
6. When it finishes, eject the install media and reboot.

## First boot

- Autologin on tty1 and the Hyprland autostart guard in `zprofile` are
  already wired up by `3-post-setup.sh` / `2-user.sh` — no manual login or
  `Hyprland` invocation needed.
- `~/.config/hypr/monitors.lua` already ships with a generic
  `preferred`/`auto`/scale-1 default (see `CLAUDE.md`, "Hyprland monitor
  config") — no per-machine setup step needed for a VM with no real
  display. A hardcoded custom resolution (e.g. to match a specific host
  laptop panel) also works even
  if the virtual GPU doesn't advertise it in `hyprctl monitors`'
  `availableModes` — Hyprland's headless/virtio-gpu backend accepts
  arbitrary custom modes regardless. If it doesn't take effect on first
  `hyprctl reload`, toggle the output off/on once
  (`hyprctl keyword monitor <name>,disable` then set the mode again) — a
  virtio-gpu quirk, not specific to this repo.
- Verify: `hyprctl monitors` should show the resolution you configured.

## Verifying GPU acceleration

`dmesg | grep -i virgl` (or `journalctl -b | grep drm`) should show
`features: +virgl`. If it doesn't, the VM's display device likely isn't set
to the `-gl` variant — recheck the hypervisor-side VM config, not the guest.

**Known non-blocking limitation**: Vulkan (`vulkaninfo --summary`) will
report `ERROR_INITIALIZATION_FAILED` unless the hypervisor exposes
Venus/blob-resource support — this is a host/hypervisor setting, not
fixable from inside the guest. It doesn't affect normal desktop use, since
Hyprland's own compositing uses GLES2/virgl, not Vulkan.
