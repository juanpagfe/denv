-- This file can be loaded by calling `lua require('plugins')` from your init.vim

-- Only required if you have packer configured as `opt`
vim.cmd [[packadd packer.nvim]]

return require('packer').startup(function(use)
    -- Packer can manage itself
    use 'wbthomason/packer.nvim'
    use "folke/tokyonight.nvim"
    use('nvim-treesitter/nvim-treesitter', { run = ':TSUpdate' })
    use('mbbill/undotree')
    use('tpope/vim-fugitive')
    use('lewis6991/gitsigns.nvim')
    use "nvim-lua/plenary.nvim"
    use "j-hui/fidget.nvim"
    use 'vimpostor/vim-tpipeline'
    use {
        'nvim-lualine/lualine.nvim',
        requires = { 'nvim-tree/nvim-web-devicons', opt = true }
    }
    use {
        'nvim-telescope/telescope.nvim', tag = '0.1.6',
        requires = { { 'nvim-lua/plenary.nvim' } }
    }
    use {
        "ThePrimeagen/harpoon",
        branch = "harpoon2",
        requires = { { "nvim-lua/plenary.nvim" } }
    }
    use {
        "danymat/neogen",
        config = function()
            require('neogen').setup {}
        end,
    }
    use "hrsh7th/nvim-cmp"
    use "hrsh7th/cmp-nvim-lsp"
    use {
        "neovim/nvim-lspconfig",
        "williamboman/mason.nvim",
        "williamboman/mason-lspconfig.nvim",
        "L3MON4D3/LuaSnip",
        "saadparwaiz1/cmp_luasnip",
        "rafamadriz/friendly-snippets"
    }
    use {
        "folke/trouble.nvim",
        cmd = "TroubleToggle", -- Lazy load when command is run
        requires = { "nvim-tree/nvim-web-devicons" },
        config = function()
            vim.api.nvim_create_autocmd("User", {
                pattern = "PackerLoad",
                callback = function(args)
                    if args.data == "folke/trouble.nvim" then
                        require("config.lazy.trouble").setup()
                    end
                end,
            })
        end
    }
end)
