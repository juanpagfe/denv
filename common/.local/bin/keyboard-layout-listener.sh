#!/usr/bin/env bash
stdbuf -oL udevadm monitor --udev --subsystem-match=input |
while read -r line; do
    if [[ "$line" =~ add[[:space:]]+.*/event[0-9]+ ]]; then
        sleep 0.5
        DISPLAY=:1 XAUTHORITY=/run/user/1000/gdm/Xauthority \
            setxkbmap -layout us -model pc105 -variant altgr-intl \
            -option compose:ralt,terminate:ctrl_alt_bksp
    fi
done
