#!/bin/bash

###############################################################################################
#                                                                                             #
#                                         GLOBAL ENV                                          #
#                                                                                             #
###############################################################################################

RED=$'\033[31m'
GREEN=$'\033[32m'
YELLOW=$'\033[33m'
BLUE=$'\033[34m'
NC=$'\033[0m'

export HOME_PATH_1000="$HOME"
export PATH="$PATH:$HOME_PATH_1000/.local/bin"
export PATH="$PATH:/opt/nvim-linux64/bin"
export PATH="$PATH:$HOME/.cargo/bin"
export XDG_DATA_DIRS="/var/lib/flatpak/exports/share:$HOME/.local/share/flatpak/exports/share:$XDG_DATA_DIRS"
export MANPAGER="nvim +Man!"
export EDITOR="nvim"

#Starts an http server on the current directory (Default port: 8000)
alias www='python3 -m http.server'

if [[ -n "${DISPLAY:-}" ]]; then
    setxkbmap -layout us -model pc105 -variant altgr-intl -option compose:ralt,terminate:ctrl_alt_bksp
fi

###############################################################################################
#                                                                                             #
#                                        GLOBAL ALIASES                                       #
#                                                                                             #
###############################################################################################


alias g='git status'
alias gaa='git add --all'
alias gcm='git commit -m'

alias ls='ls --color=auto'

#Clear terminal and change directory to home
alias c='clear'

#Close terminal
alias e='exit'

# Smart ls alias
alias l='ls -lah --color=auto'

# System
alias reboot='sudo /sbin/reboot'
alias poweroff='sudo /sbin/poweroff'
alias halt='sudo /sbin/halt'
alias shutdown='sudo /sbin/shutdown'

if command -v tmux &>/dev/null; then
    alias tmux="$(command -v tmux) -2"
fi

alias vim='nvim'
alias prime-run='env __NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia'
alias nano='nvim'
alias upgrade='sudo apt update && sudo apt upgrade -y && sudo apt autoremove -y && flatpak update -y'
alias rovo='acli rovodev'

###############################################################################################
#                                                                                             #
#                                       GLOBAL FUNCTIONS                                      #
#                                                                                             #
###############################################################################################
 
if [ -n "$BASH_VERSION" ]; then
    bind -x '"\e[24~": fzf_history_picker'   # \e[24~ = F12
elif [ -n "$ZSH_VERSION" ]; then
    zle -N fzf_history_picker
    bindkey '^[[24~' fzf_history_picker   # F12
else
    echo "Unsupported shell"
    return 1
fi

function fzf_history_picker() {
    selected=$(history \
        | awk '{$1=$2=$3=""; sub(/^ +/, ""); print}' \
        | sed '/^$/d' \
        | awk '!seen[$0]++' \
        | tac \
        | fzf --height 40% --reverse)

    if [ -n "$BASH_VERSION" ]; then
        READLINE_LINE="$selected"
        READLINE_POINT=${#READLINE_LINE}
    elif [ -n "$ZSH_VERSION" ]; then
        print -z "$selected"
    else
        echo "Unknown shell"
    fi
}

# Reloads shell
function rt() {
  exec "$SHELL" -l
}

# Display environment configuration files
catenv() {
    local name=${1:-}

    if [[ -z $name ]]; then
        name=$(
            {
                compgen -a | awk '{print "Alias\t" $0}'
                compgen -A function | awk '{print "Function\t" $0}'
                find "$HOME_PATH_1000/.local/bin" -maxdepth 1 -type f \
                    -printf 'Executable\t%f\n'
            } |
            sort -t$'\t' -k2,2 |
            fzf \
                --height=40% \
                --reverse \
                --delimiter=$'\t' \
                --with-nth=2 \
                --prompt='catenv> ' |
            cut -f2
        )

        [[ -z $name ]] && return
    fi

    if alias "$name" &>/dev/null; then
        echo "${GREEN}Alias${NC}"
        alias "$name"
    elif declare -f "$name" &>/dev/null; then
        echo "${GREEN}Function${NC}"
        declare -f "$name"
    elif [[ -f "$HOME_PATH_1000/.local/bin/$name" ]]; then
        echo "${GREEN}Executable File${NC}"
        cat "$HOME_PATH_1000/.local/bin/$name"
    else
        echo "${GREEN}Not an alias nor a function. Regex search:${NC}"
        grep "$name" /etc/envrc
    fi
}

