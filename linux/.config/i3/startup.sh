#!/bin/bash

feh --bg-scale ~/Pictures/background.jpg

if xrandr --query | grep -qE "HDMI.* connected"; then
    xrandr --output eDP-1-1 --mode 3072x1920 --scale 0.7x0.7 --pos 1920x0 --rotate normal
    xrandr --output HDMI-1-1 --primary --mode 1920x1080 --scale 1x1 --pos 0x0 --rotate normal
    i3-msg "workspace 1; move workspace to output HDMI-1-1"
    i3-msg "workspace 2; move workspace to output HDMI-1-1"
    i3-msg "workspace 10; move workspace to output eDP-1-1"
else
    xrandr --output eDP-1-1 --mode 3072x1920 --scale 0.7x0.7 --pos 0x0 --rotate normal --output HDMI-1-1 --off
    i3-msg "workspace 1; move workspace to output eDP-1-1"
    i3-msg "workspace 2; move workspace to output eDP-1-1"
fi
