local cacheFilePath = vim.fn.stdpath('data') .. '/jpcolor'


local function storecolor(color)
    local file = io.open(cacheFilePath, "w")
    if file then
        file:write(color)
        file:close()
    else
        print("Failed to open cache file for writing")
    end
end

local function getlastcolor()
    local file = io.open(cacheFilePath, "r")
    if file then
        local color = file:read("*all")
        file:close()
        vim.cmd(color)
    else
        vim.cmd("Dark");
    end
end

vim.api.nvim_create_user_command('Light', function()
    vim.opt.background = "light"
    vim.cmd("colorscheme tokyonight-day")
    vim.cmd("hi Normal guifg=#0d2573 guibg=#fffff7 ctermfg=19 ctermbg=230");
    storecolor("Light");
end, {})

vim.api.nvim_create_user_command('Dark', function()
    vim.opt.background = "dark"
    vim.cmd("colorscheme tokyonight")
    vim.cmd("hi Normal guifg=#add4fb guibg=#171421 ctermfg=19 ctermbg=230");
    storecolor("Dark");
end, {})



require("tokyonight").setup({
    -- your configuration comes here
    -- or leave it empty to use the default settings
    style = "storm",        -- The theme comes in three styles, `storm`, `moon`, a darker variant `night` and `day`
    transparent = true,     -- Enable this to disable setting the background color
    terminal_colors = true, -- Configure the colors used when opening a `:terminal` in Neovim
    styles = {
        -- Style to be applied to different syntax groups
        -- Value is any valid attr-list value for `:help nvim_set_hl`
        comments = { italic = false },
        keywords = { italic = false },
    },
})

getlastcolor()
