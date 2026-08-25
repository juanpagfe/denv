#!/bin/bash

# Build a list of available commands
cmd=$(compgen -c | sort -u | dmenu -fn "Fira Code-10" -nb "#1e1e2e" -nf "#cdd6f4" -sb "#89b4fa" -sf "#1e1e2e" -l 10 -p "Sudo: ")


if [ -n "$cmd" ]; then
    if ! command -v "$cmd" &>/dev/null; then
        notify-send "Error" "'$cmd' is not a valid command"
        exit 1
    fi
    export XAUTHORITY=~/.Xauthority
    xhost +SI:localuser:root
    pkexec env DISPLAY="$DISPLAY" XAUTHORITY="$XAUTHORITY" "$cmd"
    xhost -SI:localuser:root
fi
