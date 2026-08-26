#!/bin/sh

for id in $(xinput list --id-only 2>/dev/null); do
    if xinput list-props "$id" 2>/dev/null |
        grep -q 'libinput Tapping Enabled'; then
        xinput set-prop "$id" 'libinput Tapping Enabled' 1
    fi
done
