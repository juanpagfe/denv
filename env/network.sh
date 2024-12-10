#!/bin/bash

###############################################################################################
#                                                                                             #
#                                       NETWORK ALIASES                                       #
#                                                                                             #
###############################################################################################

#Starts an http server on the current directory (Default port: 8000)
alias www='python3 -m http.server'

#Prints current opened ports
if [ $MACHINE = "Mac" ]; then
  alias ports="netstat -lant | grep -E -i -w 'LISTEN|udp4|udp6'"
else
  alias ports='netstat -tulanp'
fi

alias lsiptables='sudo iptables -L -n -v'

###############################################################################################
#                                                                                             #
#                                      NETWORK FUNCTIONS                                      #
#                                                                                             #
###############################################################################################

# because MacOS doesn't ship with ipcalc and the brew version is crap
function is_valid_ip()
{
    local ip="$1"
    # ipcalc -c -4 -s "$ip"
    [[ "$ip" =~ ^(25[0-5]|2[0-4][0-9]|1[0-9]{2,2}|[0-9]{1,2})\.(25[0-5]|2[0-4][0-9]|1[0-9]{2,2}|[0-9]{1,2})\.(25[0-5]|2[0-4][0-9]|1[0-9]{2,2}|[0-9]{1,2})\.(25[0-5]|2[0-4][0-9]|1[0-9]{2,2}|[0-9]{1,2})$ ]]
}

function ip_of() {
    result=$(dig $1 +short)
    if [ -z "$result" ]; then
        result=$(avahi-browse -d local _ssh._tcp --resolve -t -p | grep $1 | awk --field-separator=";" '{print $8}')
    fi
    echo $result
}

#Only for linux
if [ $MACHINE = "Linux" ]; then
  #Clear IPv4 tables
  function clsiptables() {
    sudo iptables -P INPUT ACCEPT
    sudo iptables -P FORWARD ACCEPT
    sudo iptables -P OUTPUT ACCEPT
    sudo iptables -t nat -F
    sudo iptables -t mangle -F
    sudo iptables -F
    sudo iptables -X
  }

  function tunaddr() {
    sudo ifconfig $1 $2 netmask $3
  }

  # Creates a new access point
  function nap(){
    if [ -z "$1" ]; then
          echo "You need to specify a network interface in the first parameter '$1'"
          return 1
    fi
    if [ -z "$2" ]; then
          echo "You need to specify an AP SSID second parameter '$2'"
          return 1
    fi
    if [ -z "$3" ]; then
          echo "You need to specify an AP Password third parameter '$3'"
          return 1
    fi
    sudo nmcli c add type wifi ifname $1 mode ap con-name AccessPoint ssid $2
    sudo nmcli c mod AccessPoint 802-11-wireless.band bg 802-11-wireless.channel 1
    sudo nmcli c mod AccessPoint 802-11-wireless-security.key-mgmt wpa-psk 802-11-wireless-security.psk $3
    sudo nmcli c mod AccessPoint ipv4.method shared
    sudo nmcli c up AccessPoint
  }

  function open_port() {
    array=("tcp TCP udp UDP")

    re='^[0-9]+$'
    if ! [[ $1 =~ $re ]] ; then
      echo "error: $1 Not a number. Usage: open_port <protocol> <port>." >&2
      return 1
    fi

    if [[ ! " ${array[@]} " =~ " $2 " ]]; then
      echo "error: '$2' must be tcp|TCP|udp|UDP. Usage: open_port <protocol> <port>."
      return 1
    fi

    sudo iptables -t filter -A INPUT -p $2 -m state --state NEW -m $2 --dport $1 -j ACCEPT
    if [ $? -eq 0 ]; then
      echo "Port $1:$2 opened"
    else
      echo "Port $1:$2 could not be opened"
    fi
  }

    function vpn  {
        local vpnfile=~/vpn/client.ovpn
        if [ $# -gt 0 ] ; then
            case $1 in
            start)  openvpn3 session-start --config $vpnfile; return    ;;
            stop)   openvpn3 session-manage --disconnect vpn.windtalker.com --config $vpnfile; return   ;;
            list)   openvpn3 sessions-list; return  ;;
            esac
        fi
        echo "Usage $0 [start|stop|list]"
    }
fi
