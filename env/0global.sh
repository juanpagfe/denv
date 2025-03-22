#!/bin/bash

###############################################################################################
#                                                                                             #
#                                         GLOBAL ENV                                          #
#                                                                                             #
###############################################################################################

RED='\033[31m'
GREEN='\033[32m'
YELLOW='\033[33m'
BLUE='\033[34m'
NC='\033[0m'

export GOPATH=$HOME/Develop/go
export PATH=$PATH:/usr/local/go/bin:$GOPATH/bin
export HOME_PATH_1000=$(eval echo ~$USER)
export PATH=$PATH:$HOME_PATH_1000/.local/bin
export PATH="$PATH:/opt/nvim-linux64/bin"
export PATH=$PATH:/usr/local/go/bin
export PATH=$PATH:/home/jpgarcia/.cargo/bin
export XDG_DATA_DIRS="/var/lib/flatpak/exports/share:/home/jpgarcia/.local/share/flatpak/exports/share:$XDG_DATA_DIRS"

export MACHINE=$(get_machine)

#Starts an http server on the current directory (Default port: 8000)
alias www='python3 -m http.server'

#Prints current opened ports
if [[ "$MACHINE" = "Mac" ]]; then
    alias ports="netstat -lant | grep -E -i -w 'LISTEN|udp4|udp6'"
else
    alias ports='netstat -tulanp'
fi

alias lsiptables='sudo iptables -L -n -v'

setxkbmap -layout us -model pc105 -variant altgr-intl -option compose:ralt,terminate:ctrl_alt_bksp

###############################################################################################
#                                                                                             #
#                                        GLOBAL ALIASES                                       #
#                                                                                             #
###############################################################################################

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
alias mkcd='_(){ mkdir -p $1; cd $1; }; _'

# fast find
alias ff='find . -name $1'

# System
alias reboot='sudo /sbin/reboot'
alias poweroff='sudo /sbin/poweroff'
alias halt='sudo /sbin/halt'
alias shutdown='sudo /sbin/shutdown'

tmux=$(which tmux)
alias tmux="$tmux -2"

###############################################################################################
#                                                                                             #
#                                       GLOBAL FUNCTIONS                                      #
#                                                                                             #
###############################################################################################

#Create file with random base64 content
function crfile() {
  wanted_size=$(dehumanize $2)
  file_size=$((((wanted_size/12)+1)*12 ))
  read_size=$((file_size*3/4))
  dd if=/dev/urandom bs=$read_size count=1 | base64 > $1
  truncate -s "$wanted_size" $1
}

function rt() {
  exec zsh -l
}

#Setup home xrandr environment if i3 is set
function sethome() {
#    if pgrep -x "i3" > /dev/null
#    then
        xrandr --output eDP-1 --mode 1920x1200 --pos 1920x0 --rotate normal --output HDMI-1 --primary --mode 1920x1080 --pos 0x60 --rotate normal --output DP-1 --off --output DP-2 --off
#    else
#        echo "i3 is not running"
#    fi
}

#Setup alone xrandr environment if i3 is set
function setalone() {
#    if pgrep -x "i3" > /dev/null
#    then
        xrandr --output eDP-1 --primary --mode 1920x1200 --pos 0x0 --rotate normal --output HDMI-1 --off --output DP-1 --off --output DP-2 --off
#    else
#        echo "i3 is not running"
#    fi
}

# Cats environment files
function catenv() {
  if [ -z "$1" ]; then
    cat /etc/envrc
  else
    acat=$(alias | grep $1)
    if [ -z "$acat" ]; then
      fcat=$(declare -f $1)
      if [ -z "$fcat" ]; then
        if [ -f "$HOME_PATH_1000/.local/bin/$1" ]; then
            echo "${GREEN}Executable File${NC}"
            cat $HOME_PATH_1000/.local/bin/$1
        else
            echo "${GREEN}Not an alias nor a function. Regex search:${NC}"
            cat /etc/envrc | grep $1
        fi
      else
        echo "${GREEN}Function${NC}"
        echo $fcat
      fi
    else
      echo "${GREEN}Alias${NC}"
      echo "$acat"
    fi
  fi
}

function uploadenv() {
  host=$1
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

  scp /etc/envrc $1:/home/$name
  ssh $1 -T <<ENDSSH
      sudo mv ~/envrc /etc/envrc
      . /etc/envrc
      updatenv
ENDSSH
  echo "Environment shared and updated in $1"
}

function updatenv(){
  ENVRC_TEXT=". /etc/envrc"
  if [[ $(grep -L "$ENVRC_TEXT" ~/.bashrc) ]]; then
    echo $ENVRC_TEXT | sudo tee -a ~/.bashrc
  fi

  if [ -f ~/.zshrc ]; then
      if [[ $(grep -L "$ENVRC_TEXT" ~/.zshrc) ]]; then
          echo $ENVRC_TEXT | sudo tee -a ~/.zshrc
      fi
  fi

  if sudo test -f /root/.bashrc; then
    if [[ $(sudo grep -L "$ENVRC_TEXT" /root/.bashrc) ]]; then
      echo $ENVRC_TEXT | sudo tee -a /root/.bashrc
    fi
  fi

  if sudo test -f /root/.zshrc; then
    if [[ $(sudo grep -L "$ENVRC_TEXT" /root/.zshrc) ]]; then
        echo $ENVRC_TEXT | sudo tee -a /root/.zshrc
    fi
  fi

  . /etc/envrc
}

