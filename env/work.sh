#!/bin/bash

###############################################################################################
#                                                                                             #
#                                         GLOBAL ENV                                          #
#                                                                                             #
###############################################################################################

###############################################################################################
#                                                                                             #
#                                        GLOBAL ALIASES                                       #
#                                                                                             #
###############################################################################################

alias mis='cd ~/work/mis'
alias jp='cd ~/work/jp'
alias dotfiles='cd ~/work/jp/.dotfiles'
alias cnvim='cd ~/work/jp/.dotfiles/.config/nvim'

alias jrnltdy='jrnl -on today --edit'
alias jrnlon='jrnl --edit -on'

export DYNAPP_SERVER="165.227.221.238"
export GANAD_SERVER="64.225.48.51"
export QAADMIN_DIR="/Users/juanpablogarcia/work/mis/qaadmin-be"
export CARDIACQA_DIR="/Users/juanpablogarcia/work/mis/cardiacqa-be"


###############################################################################################
#                                                                                             #
#                                       GLOBAL FUNCTIONS                                      #
#                                                                                             #
###############################################################################################

function cardiacqa_new_migration() {
    if [ -z "$1" ]; then
        echo -e "You must give the name of the new migration"
        return 1
    fi
    dotnet ef migrations add $1 --startup-project $CARDIACQA_DIR/CardiacQA.App/CardiacQA.App.csproj --project $QAADMIN_DIR/QAAdmin.DataAccess/CardiacQA.DataAccess.csproj --context CardiacContext
} 

function cardiacqa_update_migrations() {
    dotnet ef database update --startup-project $CARDIACQA_DIR/CardiacQA.App/CardiacQA.App.csproj --project $QAADMIN_DIR/QAAdmin.DataAccess/CardiacQA.DataAccess.csproj --context CardiacContext
} 

function cardiacqa_run() {
    dotnet run --project $CARDIACQA_DIR/CardiacQA.App/CardiacQA.App.csproj
} 

function qaadmin_new_migration() {
    if [ -z "$1" ]; then
        echo -e "You must give the name of the new migration"
        return 1
    fi
    dotnet ef migrations add $1 --startup-project $QAADMIN_DIR/QAAdmin.API/QAAdmin.API.csproj --project $QAADMIN_DIR/QAAdmin.DataAccess/QAAdmin.DataAccess.csproj --context ApplicationDbContext
} 

function qaadmin_update_migrations() {
    dotnet ef database update --startup-project $QAADMIN_DIR/QAAdmin.API/QAAdmin.API.csproj --project $QAADMIN_DIR/QAAdmin.DataAccess/QAAdmin.DataAccess.csproj --context ApplicationDbContext
} 

function qaadmin_run() {
    dotnet run --project $QAADMIN_DIR/QAAdmin.API/QAAdmin.API.csproj
} 
