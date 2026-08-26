-- Pull in the wezterm API
local wezterm = require 'wezterm'

-- This will hold the configuration.
local config = wezterm.config_builder()

-- This is where you actually apply your config choices.
config.enable_tab_bar = false

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
config.font_size = 10
config.color_scheme = "Tokyo Night"

config.term = "xterm-256color"
config.scrollback_lines = 100000
config.audible_bell = "Disabled"
config.enable_wayland = false
config.window_close_confirmation = "NeverPrompt"
config.adjust_window_size_when_changing_font_size = false
config.warn_about_missing_glyphs = false

-- Loop from 1 to 9 to create tmux window switchers
config.keys = {}
for i = 1, 9 do
  table.insert(config.keys, {
    key = tostring(i),
    mods = 'ALT',
    -- Sends Ctrl-a (tmux prefix) followed by the number
    action = wezterm.action.SendString('\x01' .. tostring(i)),
  })
end

-- Finally, return the configuration to wezterm:
return config
