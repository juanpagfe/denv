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

export GOPATH="$HOME/Develop/go"
export PATH="$PATH:/usr/local/go/bin:$GOPATH/bin"
export HOME_PATH_1000="$HOME"
export PATH="$PATH:$HOME_PATH_1000/.local/bin"
export PATH="$PATH:/opt/nvim-linux64/bin"
export PATH="$PATH:$HOME/.cargo/bin"
export XDG_DATA_DIRS="/var/lib/flatpak/exports/share:$HOME/.local/share/flatpak/exports/share:$XDG_DATA_DIRS"
export TERM=xterm-256color
export MANPAGER="nvim +Man!"

#Starts an http server on the current directory (Default port: 8000)
alias www='python3 -m http.server'

alias lsiptables='sudo iptables -L -n -v'

if [[ -n "${DISPLAY:-}" ]]; then
    setxkbmap -layout us -model pc105 -variant altgr-intl -option compose:ralt,terminate:ctrl_alt_bksp
fi

###############################################################################################
#                                                                                             #
#                                        GLOBAL ALIASES                                       #
#                                                                                             #
###############################################################################################


alias stream-android='scrcpy'

alias ls='ls --color=auto'

#Clear terminal and change directory to home
alias c='clear'

#Creates a file
alias t='touch'

#Close terminal
alias e='exit'

#History+grep shortcut
alias hs='history | grep'

# Smart ls alias
alias l='ls -lah'

# Make and change directory at once
mkcd() { mkdir -p "$1" && cd "$1"; }

# fast find
ff() { find . -name "$1"; }

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

#Create file with random base64 content
function crfile() {
  wanted_size=$(dehumanize "$2")
  file_size=$((((wanted_size/12)+1)*12 ))
  read_size=$((file_size*3/4))
  dd if=/dev/urandom bs="$read_size" count=1 2>/dev/null | base64 > "$1"
  truncate -s "$wanted_size" "$1"
}

function rt() {
  exec "$SHELL" -l
}

#Setup home xrandr environment (requires i3)
function sethome() {
    if ! pgrep -x "i3" > /dev/null; then
        echo "i3 is not running"
        return 1
    fi
    xrandr --output eDP-1-1 --mode 3072x1920 --scale 0.7x0.7 --pos 1920x0 --rotate normal --output HDMI-1-1 --primary --mode 1920x1080 --pos 0x0 --rotate normal
}

#Setup alone xrandr environment (requires i3)
function setalone() {
    if ! pgrep -x "i3" > /dev/null; then
        echo "i3 is not running"
        return 1
    fi
    xrandr --output eDP-1-1 --mode 3072x1920 --scale 0.7x0.7 --pos 0x0 --rotate normal --output HDMI-1-1 --off --output DP-1 --off
}

# Display environment configuration files
function catenv() {
  if [ -z "$1" ]; then
    cat /etc/envrc
  else
    acat=$(alias | grep "$1")
    if [ -z "$acat" ]; then
      fcat=$(declare -f "$1")
      if [ -z "$fcat" ]; then
        if [ -f "$HOME_PATH_1000/.local/bin/$1" ]; then
            echo "${GREEN}Executable File${NC}"
            cat "$HOME_PATH_1000/.local/bin/$1"
        else
            echo "${GREEN}Not an alias nor a function. Regex search:${NC}"
            grep "$1" /etc/envrc
        fi
      else
        echo "${GREEN}Function${NC}"
        echo "$fcat"
      fi
    else
      echo "${GREEN}Alias${NC}"
      echo "$acat"
    fi
  fi
}

function uploadenv() {
  host="$1"
  name="${1%@*}"
  if [ -z "$host" ]; then
    echo "You need to specify the host (eg. pi@pi0.local)"
    return
  fi

  if [ -z "$name" ]; then
    echo "You need to specify the username (eg. pi@pi0.local. It can't be root)"
    return
  fi

  if [ "$name" = "root" ]; then
    echo "You need to specify the username (eg. pi@pi0.local. It can't be root)"
    return
  fi

  scp /etc/envrc "$host:/home/$name"
  ssh "$host" -T <<ENDSSH
      sudo mv ~/envrc /etc/envrc
      . /etc/envrc
      updatenv
ENDSSH
  echo "Environment shared and updated in $host"
}

function updatenv(){
  ENVRC_TEXT=". /etc/envrc"
  if ! grep -qF "$ENVRC_TEXT" ~/.bashrc 2>/dev/null; then
    echo "$ENVRC_TEXT" | sudo tee -a ~/.bashrc
  fi

  if [ -f ~/.zshrc ]; then
      if ! grep -qF "$ENVRC_TEXT" ~/.zshrc 2>/dev/null; then
          echo "$ENVRC_TEXT" | sudo tee -a ~/.zshrc
      fi
  fi

  if sudo test -f /root/.bashrc; then
    if ! sudo grep -qF "$ENVRC_TEXT" /root/.bashrc 2>/dev/null; then
      echo "$ENVRC_TEXT" | sudo tee -a /root/.bashrc
    fi
  fi

  if sudo test -f /root/.zshrc; then
    if ! sudo grep -qF "$ENVRC_TEXT" /root/.zshrc 2>/dev/null; then
        echo "$ENVRC_TEXT" | sudo tee -a /root/.zshrc
    fi
  fi

  . /etc/envrc
}

