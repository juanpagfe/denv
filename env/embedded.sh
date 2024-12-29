#!/bin/bash

###############################################################################################
#                                                                                             #
#                                         GLOBAL ENV                                          #
#                                                                                             #
###############################################################################################
export ESP_IDF_PATH=/opt/esp-idf
export DEF_ESP_CHIP="esp32c3"
export DEF_ESP_PORT=/dev/ttyACM0

function idf() {
    if [ -z "$IDF_PATH" ]; then
        echo "You need to load esp-idf framework before using this program."
        echo "Run:"
        echo "  source $ESP_IDF_PATH/export.sh"
        return 1
    fi
    idf.py $@
}
