#!/bin/bash

# Get all .desktop entries
apps=$(find /usr/share/applications ~/.local/share/applications -name "*.desktop" 2>/dev/null)

# Parse names and commands
choices=$(while IFS= read -r desktop; do
    name=$(grep -m 1 '^Name=' "$desktop" | cut -d= -f2)
    exec=$(grep -m 1 '^Exec=' "$desktop" | cut -d= -f2 | sed 's/ *%[fFuUdDnNickvm]//g')
    if [[ -n "$name" && -n "$exec" ]]; then
        echo "$name###$exec"
    fi
done <<< "$apps" | sort)

# Show in dmenu
selected=$(echo "$choices" | cut -d'#' -f1 | dmenu -i -l 10 -p "Run:")

# Extract command
cmd=$(echo "$choices" | grep "^$selected###" | cut -d'#' -f4-)

# Run if not empty
echo "$cmd"
[[ -n "$cmd" ]] && eval "$cmd" &
