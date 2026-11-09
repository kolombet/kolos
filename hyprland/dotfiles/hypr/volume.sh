#!/bin/bash
# Per-sink volume step, called by bindings.lua's XF86AudioLowerVolume/
# RaiseVolume. The audible range isn't the same fraction of PipeWire's
# software volume scale on every output: the Sony wireless USB headset's
# analog amp maxes out by ~0.10, while the built-in speaker/headphone jack
# (CS4208, after the model=mbp11 kernel fixup - see ~/sound-doc.md) uses the
# full 0.0-1.0 range. kolos-bar's stats.rs volume() must use the same
# per-sink cap to rescale its displayed percentage correctly.
SONY_HEADSET_SINK="alsa_output.usb-Sony_Interactive_Entertainment_Wireless_Headset-00.analog-stereo"

if [ "$(pactl get-default-sink)" = "$SONY_HEADSET_SINK" ]; then
    cap=0.10
else
    cap=1.0
fi

case "$1" in
    up) wpctl set-volume -l "$cap" @DEFAULT_AUDIO_SINK@ 1%+ ;;
    down) wpctl set-volume -l "$cap" @DEFAULT_AUDIO_SINK@ 1%- ;;
    *) echo "usage: volume.sh up|down" >&2; exit 1 ;;
esac
