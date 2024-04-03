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

unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     export MACHINE=Linux;;
    Darwin*)    export MACHINE=Mac;;
    CYGWIN*)    export MACHINE=Cygwin;;
    MINGW*)     export MACHINE=MinGw;;
    *)          export MACHINE="UNKNOWN:${unameOut}"
esac


###############################################################################################
#                                                                                             #
#                                        GLOBAL ALIASES                                       #
#                                                                                             #
###############################################################################################

YUM_CMD=$(which yum >> /dev/null; echo $?)
APT_GET_CMD=$(which apt-get >> /dev/null; echo $?)
BREW_GET_CMD=$(which brew >> /dev/null; echo $?)

if [[ $YUM_CMD -eq 0 ]]; then
  export PACKAGE_MANAGER=yum
elif [[ $APT_GET_CMD -eq 0 ]]; then
  export PACKAGE_MANAGER=apt-get
elif [[ $BREW_CMD -eq 0 ]]; then
  export PACKAGE_MANAGER=brew
else
  echo "Error, can't get PMAN"
fi

#Package manager
function pman(){
  if [ $PACKAGE_MANAGER = "brew" ]; then
    $PACKAGE_MANAGER $@
  else
    sudo $PACKAGE_MANAGER $@
  fi
}

#Clear terminal and change directory to home
alias c='clear'

if [ $MACHINE = "Mac" ]; then
  alias copy="pbcopy"
else
  alias copy="xclip -i -sel p -f | xclip -i -sel c"
fi

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

alias vim='nvim'

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

# Reloads environment scripts on the current session
function reload_scripts() {
  source /etc/envrc 
}


function rt() {
  exec zsh -l
}


# Extract any kind of know files
function extract() {

  if [[ "$#" -lt 1 ]]; then
    echo "Usage: extract <path/file_name>.<zip|rar|bz2|gz|tar|tbz2|tgz|Z|7z|xz|ex|tar.bz2|tar.gz|tar.xz>"
    return 1 #not enough args
  fi

  if [[ ! -e "$1" ]]; then
    echo -e "File does not exist!"
    return 2 # File not found
  fi

  DESTDIR="."

  filename=$(basename "$1")

  case "${filename##*.}" in
  tar)
    echo -e "Extracting $1 to $DESTDIR: (uncompressed tar)"
    tar xvf "$1" -C "$DESTDIR"
    ;;
  gz)
    echo -e "Extracting $1 to $DESTDIR: (gip compressed tar)"
    tar xvfz "$1" -C "$DESTDIR"
    ;;
  tgz)
    echo -e "Extracting $1 to $DESTDIR: (gip compressed tar)"
    tar xvfz "$1" -C "$DESTDIR"
    ;;
  xz)
    echo -e "Extracting  $1 to $DESTDIR: (gip compressed tar)"
    tar xvf -J "$1" -C "$DESTDIR"
    ;;
  bz2)
    echo -e "Extracting $1 to $DESTDIR: (bzip compressed tar)"
    tar xvfj "$1" -C "$DESTDIR"
    ;;
  tbz2)
    echo -e "Extracting $1 to $DESTDIR: (tbz2 compressed tar)"
    tar xvjf "$1" -C "$DESTDIR"
    ;;
  zip)
    echo -e "Extracting $1 to $DESTDIR: (zipp compressed file)"
    unzip "$1" -d "$DESTDIR"
    ;;
  lzma)
    echo -e "Extracting $1 : (lzma compressed file)"
    unlzma "$1"
    ;;
  rar)
    echo -e "Extracting $1 to $DESTDIR: (rar compressed file)"
    unrar x "$1" "$DESTDIR"
    ;;
  7z)
    echo -e "Extracting $1 to $DESTDIR: (7zip compressed file)"
    7za e "$1" -o "$DESTDIR"
    ;;
  xz)
    echo -e "Extracting $1 : (xz compressed file)"
    unxz "$1"
    ;;
  exe)
    cabextract "$1"
    ;;
  *)
    echo -e "Unknown archieve format!"
    return
    ;;
  esac
}

#Compress file. Only tar and zip files supported
function compress() {
  if [[ "$#" -lt 1 ]]; then
    echo "Usage: compress <path/file_name>.<zip|rar|bz2|gz|tar|tbz2|tgz|Z|7z|xz|ex|tar.bz2|tar.gz|tar.xz> <path/file_names>"
    return 1 #not enough args
  fi

  filename=$(basename "$1")
  arr=("$@")
  files="${arr[@]:1}"

  case "${filename##*.}" in
  zip)
    echo -e "Compressing $files to $1: (${filename##*.} file)"
    zip -r $1 $files
    ;;
  tar)
    echo -e "Extracting $1 to $DESTDIR: (${filename##*.} tar)"
    tar -cavvf $1 $files
    ;;
  gz)
    echo -e "Extracting $1 to $DESTDIR: (${filename##*.} compressed tar)"
    tar -cavvf $1 $files
    ;;
  tgz)
    echo -e "Extracting $1 to $DESTDIR: (${filename##*.} compressed tar)"
    tar -cavvf $1 $files
    ;;
  xz)
    echo -e "Extracting  $1 to $DESTDIR: (${filename##*.} compressed tar)"
    tar -cavvf $1 $files
    ;;
  bz2)
    echo -e "Extracting $1 to $DESTDIR: (${filename##*.} compressed tar)"
    tar -cavvf $1 $files
    ;;
  tbz2)
    echo -e "Extracting $1 to $DESTDIR: (${filename##*.} compressed tar)"
    tar -cavvf $1 $files
    ;;
  *)
    echo -e "Unknown archieve format!"
    return
    ;;
  esac
}


#Setup home xrandr environment if i3 is set
function sethome() {
    if pgrep -x "i3" > /dev/null
    then
        xrandr --output eDP-1 --mode 1920x1200 --pos 1920x0 --rotate normal --output HDMI-1 --primary --mode 1920x1080 --pos 0x60 --rotate normal --output DP-1 --off --output DP-2 --off
    else
        echo "i3 is not running"
    fi
}

#Setup alone xrandr environment if i3 is set
function setalone() {
    if pgrep -x "i3" > /dev/null
    then
        xrandr --output eDP-1 --primary --mode 1920x1200 --pos 0x0 --rotate normal --output HDMI-1 --off --output DP-1 --off --output DP-2 --off
    else
        echo "i3 is not running"
    fi
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
        echo "${GREEN}Not an alias nor a function. Regex search:${NC}"
        cat /etc/envrc | grep $1
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
