#!/bin/bash

###############################################################################################
#                                                                                             #
#                                        GIT ALIASES                                          #
#                                                                                             #
###############################################################################################

# Git status alias
alias g='git status -sb'

# Git add and remove aliases
alias ga='git add'
alias gr='git rm'

# Git commit aliases
alias gcm='git commit -m'

# Git checkout aliases
alias gcob='git checkout -b'
alias gcof='git checkout'
alias gsw='git switch'

# Git fetch aliases
alias gf='git fetch'
alias gfa='git fetch --all'

# Git pull alias
alias gp='git pull --rebase'

# Git pull from origin aliases
alias gprod='git pull --rebase origin dev'
alias gprom='git pull --rebase origin main'

# Git push to origin aliases
alias gpush='git push origin'
alias gpod='git push origin dev'
alias gpom='git push origin master'

# Git push to upstream aliases
alias gpud='git push upstream dev'
alias gpum='git push upstream master'

# Git stash aliases
alias gsl='git stash list'
alias gsp='git stash pop'
alias gss='git stash save'

# Git diff and log aliases
alias gd='git diff --color-words'
alias gl='git log --oneline --decorate'
alias gslog="git log --graph --abbrev-commit --decorate --date=relative --format=format:'%C(bold blue)%h%C(reset) - %C(bold green)(%ar)%C(reset) %C(white)%s%C(reset) %C(dim white)- %an%C(reset)%C(bold yellow)%d%C(reset)' --all"

# Iterate through git submodules
alias gsfe='git submodule foreach'

