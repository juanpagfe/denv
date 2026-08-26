#!/bin/bash

chosen=$(printf '%s\n' \
    " Poweroff" \
    " Restart" \
    " Logout" |
    rofi -dmenu -i -p "System")

[[ -z "$chosen" ]] && exit 0

confirm=$(printf '%s\n' "No" "Yes" |
    rofi -dmenu -i -p "Are you sure?")

[[ "$confirm" != "Yes" ]] && exit 0

case "$chosen" in
    " Poweroff") systemctl poweroff ;;
    " Restart")  systemctl reboot ;;
    " Logout")   i3-msg exit ;;
esac
