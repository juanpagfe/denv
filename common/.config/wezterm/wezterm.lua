-- Pull in the wezterm API
local wezterm = require 'wezterm'

-- This will hold the configuration.
local config = wezterm.config_builder()

-- This is where you actually apply your config choices.
config.enable_tab_bar = false
config.default_gui_startup_args = { "start", "--maximize" }

config.window_padding = {
  left = 5,
  right = 0,
  top = 5,
  bottom = 0,
}

config.font = wezterm.font_with_fallback {
  { family = "JetBrains Mono", weight = "Medium" },
}

-- or, changing the font size and color scheme.
config.font_size = 11
config.color_scheme = "Tokyo Night"

config.term = "wezterm"
config.scrollback_lines = 100000
config.audible_bell = "Disabled"

-- Finally, return the configuration to wezterm:
return config
