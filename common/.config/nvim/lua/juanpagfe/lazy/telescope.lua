return {
    "nvim-telescope/telescope.nvim",
    tag = "0.1.5",
    dependencies = {
        "nvim-lua/plenary.nvim",
    },
    config = function()
        require('telescope').setup({
            defaults = {
                file_ignore_patterns = {"%.git/", "node_modules/"},
            },
            pickers = {
                find_files = {
                    hidden = true,
                },
            }
        })

        local builtin = require('telescope.builtin')

        -- Keybindings for Telescope
        vim.keymap.set('n', '<C-p>', function()
            builtin.find_files({ hidden = true })
        end)
        vim.keymap.set('n', '<leader>pf', function()
            builtin.find_files({ hidden = false })
        end)
        vim.keymap.set('n', '<leader>ph', builtin.git_files, {})
        vim.keymap.set('n', '<leader>ps', function()
            builtin.live_grep({
                additional_args = function()
                    return { "--hidden" }
                end,
            })
        end, {})
        vim.api.nvim_set_keymap('n', '<leader><Tab>', ":lua require('telescope.builtin').buffers { sort_lastused = true, ignore_current_buffer = true, only_cwd = true }<CR>", { noremap = true, silent = true })
        vim.keymap.set('n', '<leader>r', function()
            builtin.lsp_references({ include_declaration = true, include_current_line = true })
        end)
    end,
}
