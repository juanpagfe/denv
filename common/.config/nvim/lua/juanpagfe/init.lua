-- Load various configuration files for settings, key mappings, commands, and lazy initialization
require("juanpagfe.set")       -- Load the 'set' configuration (likely settings like `vim.opt`)
require("juanpagfe.remap")      -- Load the 'remap' configuration (likely key mappings)
require("juanpagfe.cmd")        -- Load the 'cmd' configuration (likely custom commands)
require("juanpagfe.lazy_init")  -- Load the 'lazy_init' configuration (likely initialization of lazy-loaded modules)

-- Create an augroup for organizing autocmds related to the 'juanpagfe' configuration
local augroup = vim.api.nvim_create_augroup
local juanpagfeGroup = augroup('juanpagfe', {})

-- Create an augroup for highlighting yank operations
local autocmd = vim.api.nvim_create_autocmd
local yank_group = augroup('HighlightYank', {})

-- Define a function to reload a module using plenary (typically used for reloading Neovim plugins/modules)
function R(name)
    require("plenary.reload").reload_module(name)  -- Reload the specified module
end

-- Register a new filetype based on the 'templ' file extension
vim.filetype.add({
    extension = {
        templ = 'templ',  -- Associate files with `.templ` extension to the 'templ' filetype
    }
})

-- Autocmd to highlight the yanked text with a specified highlight group after text yank
autocmd('TextYankPost', {
    group = yank_group,          -- Group the autocmd under the 'HighlightYank' group
    pattern = '*',                -- Apply to all files
    callback = function()         -- Define what happens after text is yanked
        vim.highlight.on_yank({
            higroup = 'IncSearch',  -- Use the 'IncSearch' highlight group
            timeout = 40,            -- Highlight will stay for 40ms
        })
    end,
})

-- Autocmd to remove trailing whitespaces before saving a file
autocmd({"BufWritePre"}, {
    group = juanpagfeGroup,       -- Group the autocmd under the 'juanpagfe' group
    pattern = "*",                -- Apply to all files
    command = [[%s/\s\+$//e]],     -- Remove trailing spaces from the entire file before saving
})

-- Autocmd to set a colorscheme depending on the file type when entering a buffer
autocmd('BufEnter', {
    group = juanpagfeGroup,        -- Group the autocmd under the 'juanpagfe' group
    callback = function()          -- Define what happens when entering a buffer
        vim.cmd.colorscheme("tokyonight-night")
    end
})

-- Autocmd to set up LSP key mappings when an LSP server is attached to the buffer
autocmd('LspAttach', {
    group = juanpagfeGroup,        -- Group the autocmd under the 'juanpagfe' group
    callback = function(e)         -- Define key mappings when LSP server is attached
        local opts = { buffer = e.buf }  -- Define buffer-specific options for keymaps
        -- Map 'gd' to go to the definition of the symbol under the cursor
        vim.keymap.set("n", "gd", function() vim.lsp.buf.definition() end, opts)
        -- Map 'K' to show hover information for the symbol under the cursor (e.g., documentation)
        vim.keymap.set("n", "K", function() vim.lsp.buf.hover() end, opts)
        -- Map '<leader>vws' to search for a workspace symbol (e.g., find a symbol across the entire workspace)
        vim.keymap.set("n", "<leader>vws", function() vim.lsp.buf.workspace_symbol() end, opts)
        -- Map '<leader>vd' to open a floating window with diagnostics for the current buffer
        vim.keymap.set("n", "<leader>vd", function() vim.diagnostic.open_float() end, opts)
        -- Map '<leader>vca' to trigger a code action (e.g., quick fixes, refactoring)
        vim.keymap.set("n", "<leader>vca", function() vim.lsp.buf.code_action() end, opts)
        -- Map '<leader>vrr' to show references for the symbol under the cursor
        vim.keymap.set("n", "<leader>vrr", function() vim.lsp.buf.references() end, opts)
        -- Map '<leader>vrn' to rename the symbol under the cursor
        vim.keymap.set("n", "<leader>vrn", function() vim.lsp.buf.rename() end, opts)
        -- Map '<C-h>' in insert mode to show signature help for the symbol under the cursor
        vim.keymap.set("i", "<C-h>", function() vim.lsp.buf.signature_help() end, opts)
        -- Map '[d' to jump to the next diagnostic (error/warning) in the current buffer
        vim.keymap.set("n", "[d", function() vim.diagnostic.goto_next() end, opts)
        -- Map ']d' to jump to the previous diagnostic (error/warning) in the current buffer
        vim.keymap.set("n", "]d", function() vim.diagnostic.goto_prev() end, opts)
    end
})

-- Configure netrw (the built-in file explorer) settings:
vim.g.netrw_browse_split = 0  -- Open directories in the current window, not in a new one
vim.g.netrw_banner = 1        -- Disable the netrw banner (the header with version info)
vim.g.netrw_winsize = 25      -- Set the default window size for netrw to 25% of the screen

