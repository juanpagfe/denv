#!/bin/bash

chosen=$(echo -e " Poweroff\n Restart\n Logout" | dmenu -l 3 -i -p "System")

# Exit if no option was selected
if [[ -z "$chosen" ]]; then
    exit 0
fi

confirm=$(echo -e "No\nYes" | dmenu -l 2 -i -p "Are you sure?")

if [[ "$confirm" == "Yes" ]]; then
    case "$chosen" in
        " Poweroff") systemctl poweroff ;;
        " Restart") systemctl reboot ;;
        " Logout") i3-msg exit ;;
    esac
fi
