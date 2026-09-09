#!/usr/bin/env bash

window_id=$(
    i3-msg -t get_tree |
        jq -r '
            .. |
            objects |
            select(
                .window_properties? and
                .window_properties.class? == "zen"
            ) |
            .window
        ' |
        head -n1
)

[ -n "$window_id" ] || exit 0

wmctrl -i -r "$window_id" -b add,demands_attention
