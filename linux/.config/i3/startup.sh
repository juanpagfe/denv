#!/bin/bash

feh --bg-scale ~/Pictures/background.jpg

if xrandr --query | grep -qE "HDMI.* connected"; then
    xrandr --output eDP-1 --mode 1920x1200 --pos 1920x0 --rotate normal --output HDMI-1 --primary --mode 1920x1080 --pos 0x60 --rotate normal --output DP-1 --off --output DP-2 --off
    declare -A screen_workspaces=(
        ["DP-1"]="10"
        ["HDMI-1"]="1 2"
    )
    declare -A workspace_apps=(
        ["1"]="alacritty"
        ["2"]="google-chrome"
        ["10"]="obsidian"
    )
else
    xrandr --output eDP-1 --primary --mode 1920x1200 --pos 0x0 --rotate normal --output HDMI-1 --off --output DP-1 --off --output DP-2 --off
    declare -A screen_workspaces=()
    declare -A workspace_apps=(
        ["1"]="alacritty"
        ["2"]="google-chrome"
        ["3"]="obsidian"
    )
fi

# Assign each workspace to the appropriate screen
for screen in "${!screen_workspaces[@]}"; do
    for workspace in ${screen_workspaces[$screen]}; do
        # Move the workspace to the specified screen
        i3-msg workspace $workspace
        i3-msg move workspace to output $screen
        sleep 0.5
    done
done

for workspace in "${!workspace_apps[@]}"; do
    # Switch to the workspace
    i3-msg workspace $workspace
    
    # Launch the application
    ${workspace_apps[$workspace]} &
    
    # Add a brief delay to ensure the application launches in the correct workspace
    sleep 0.5
done
