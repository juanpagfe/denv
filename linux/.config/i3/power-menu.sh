#!/bin/bash
chosen=$(echo -e " Poweroff\n Restart\n Logout" | rofi -dmenu -i -p "System")

if [[ -z "$chosen" ]]; then
    exit 0
fi

confirm=$(echo -e "No\nYes" | rofi -dmenu -p "Are you sure?")

if [[ "$confirm" == "Yes" ]]; then
    case "$chosen" in
        " Poweroff") systemctl poweroff ;;
        " Restart") systemctl reboot ;;
        " Logout") i3-msg exit ;;
    esac
fi
