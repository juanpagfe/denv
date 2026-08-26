#!/bin/sh

if pgrep -x pavucontrol >/dev/null; then
    wmctrl -xa pavucontrol.Pavucontrol
else
    pavucontrol &
fi
