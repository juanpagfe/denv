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

-- or, changing the font size and color scheme.
config.font_size = 11
config.color_scheme = 'Afterglow'

-- Finally, return the configuration to wezterm:
return config
