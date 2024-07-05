#!/bin/zsh

#Configure

if [ ! -d ~/.config ]; then
    mkdir ~/.config
fi
if [ "$machine" = "Mac" ]; then
    cp -rf mac/.config/* ~/.config
fi
if [ "$machine" = "Linux" ]; then
    cp -rf linux/.config/* ~/.config
fi
cp -rf common/.config/* ~/.config


#Copy bin
if [ ! -d ~/.local ]; then
    mkdir ~/.local
fi
cp -r common/.local/bin/* ~/.local/bin

#Merge environment files
if [ ! -f envrc ]; then
    echo "Generating envrc file"
    if [ -f /tmp/envrc ]; then
        sudo rm /tmp/envrc
    fi
    touch /tmp/envrc
    cat env/*.sh > /tmp/envrc
fi

#Install environment files
if [ -f envrc ]; then
    sudo mv envrc /etc/envrc
else
    sudo mv /tmp/envrc /etc/envrc
fi
echo "Installation completed"
