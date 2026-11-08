#!/usr/bin/env python3
# Re-runs randomwallpaper.sh when a monitor is plugged or unplugged: swaybg only covers the
# outputs that existed when it started.
import os, socket, subprocess, time

sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(os.path.join(os.environ["XDG_RUNTIME_DIR"], "hypr",
                          os.environ["HYPRLAND_INSTANCE_SIGNATURE"], ".socket2.sock"))
script = os.path.expanduser("~/.config/hypr/randomwallpaper.sh")
buf = b""
while chunk := sock.recv(4096):
    buf += chunk
    *lines, buf = buf.split(b"\n")
    if any(l.startswith((b"monitoraddedv2>>", b"monitorremoved>>")) for l in lines):
        time.sleep(1)   # let the new output finish its first modeset
        subprocess.Popen([script], start_new_session=True)
