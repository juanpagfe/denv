#!/bin/bash

cmd=$(
    compgen -c |
        sort -u |
        rofi -dmenu \
            -p "Sudo:" \
            -lines 10
)

if [[ -z "$cmd" ]]; then
    exit 0
fi

if ! command -v "$cmd" &>/dev/null; then
    notify-send "Error" "'$cmd' is not a valid command"
    exit 1
fi

export XAUTHORITY="$HOME/.Xauthority"

xhost +SI:localuser:root
pkexec env \
    DISPLAY="$DISPLAY" \
    XAUTHORITY="$XAUTHORITY" \
    "$cmd"
xhost -SI:localuser:root
