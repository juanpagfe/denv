return {
  "nvim-telescope/telescope.nvim",
  tag = "0.1.5",
  dependencies = {
    "nvim-lua/plenary.nvim",
  },
  config = function()
    require('telescope').setup({})

    local builtin = require('telescope.builtin')

    -- Keybindings for Telescope
    vim.keymap.set('n', '<C-p>', function()
      builtin.find_files({ hidden = true })
    end)
    vim.keymap.set('n', '<leader>pf', function()
      builtin.find_files({ hidden = false })
    end)
    vim.keymap.set('n', '<leader>ph', builtin.git_files, {})
    vim.keymap.set('n', '<leader>ps', builtin.live_grep, {})
    vim.keymap.set('n', '<leader>pb', builtin.buffers, {})
    vim.keymap.set('n', '<leader>gr', ":TelescopeGrepReplace ", { noremap = true, silent = true })
    vim.keymap.set('n', '<leader>r', function()
      builtin.lsp_references({ include_declaration = true, include_current_line = true })
    end)
  end,
}
