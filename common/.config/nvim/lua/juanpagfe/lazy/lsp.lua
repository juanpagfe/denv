return {
    "neovim/nvim-lspconfig",
    dependencies = {
        "stevearc/conform.nvim",
        "williamboman/mason.nvim",
        "williamboman/mason-lspconfig.nvim",
        "hrsh7th/nvim-cmp",
        "hrsh7th/cmp-nvim-lsp",
        "L3MON4D3/LuaSnip",
        "saadparwaiz1/cmp_luasnip",
        "windwp/nvim-ts-autotag"
    },
    config = function()
        local cmp = require('cmp')
        local luasnip = require("luasnip")
        require("mason").setup()
        require("mason-lspconfig").setup({ ensure_installed = { "lua_ls", "solargraph" } })
        require("nvim-ts-autotag").setup()
        require("luasnip.loaders.from_vscode").lazy_load()

        local lspconfig = require("lspconfig")
        local cmp_nvim_lsp = require("cmp_nvim_lsp")
        local capabilities = cmp_nvim_lsp.default_capabilities()

        local servers = { "ts_ls", "html", "cssls", "jsonls", "clangd", "pyright", "eslint", "groovyls", "rust_analyzer" }
        for _, server in ipairs(servers) do
            if server == "groovyls" then
                lspconfig[server].setup({
                    capabilities = capabilities,
                    filetypes = { "groovy", "Jenkinsfile" }
                })
            else
                lspconfig[server].setup({ capabilities = capabilities })
            end
        end

        lspconfig.lua_ls.setup({
            settings = {
                Lua = {
                    diagnostics = { globals = { "vim" } },
                    workspace = {
                        library = { [vim.fn.expand("$VIMRUNTIME/lua")] = true }
                    }
                }
            }
        })

        lspconfig.emmet_ls.setup({
            capabilities = capabilities,
            filetypes = { "html", "javascriptreact", "typescriptreact" }
        })

        vim.api.nvim_create_autocmd("LspAttach", {
            callback = function(ev)
                local opts = { buffer = ev.buf }
                local keymap = vim.keymap.set
                keymap("n", "gD", vim.lsp.buf.declaration, opts)
                keymap("n", "gd", vim.lsp.buf.definition, opts)
                keymap("n", "K", vim.lsp.buf.hover, opts)
                keymap("n", "gi", vim.lsp.buf.implementation, opts)
                keymap("n", "<space>wa", vim.lsp.buf.add_workspace_folder, opts)
                keymap("n", "<space>wr", vim.lsp.buf.remove_workspace_folder, opts)
                keymap("n", "<space>wl", function() print(vim.inspect(vim.lsp.buf.list_workspace_folders())) end, opts)
                keymap("n", "<space>D", vim.lsp.buf.type_definition, opts)
                keymap("n", "<space>rn", vim.lsp.buf.rename, opts)
                keymap({ "n", "v" }, "<space>ca", vim.lsp.buf.code_action, opts)
                keymap("n", "gr", vim.lsp.buf.references, opts)
                keymap("n", "<space>f", function() vim.lsp.buf.format { async = true } end, opts)
            end
        })

        vim.diagnostic.config({
            float = {
                focusable = false,
                style = "minimal",
                border = "rounded",
                source = "always",
                header = "",
                prefix = "",
            },
        })

        local has_words_before = function()
            local line, col = table.unpack(vim.api.nvim_win_get_cursor(0))
            return col ~= 0 and vim.api.nvim_buf_get_lines(0, line - 1, line, true)[1]:sub(col, col):match("%s") == nil
        end

        cmp.setup({
            mapping = cmp.mapping.preset.insert({
                ["<Tab>"] = cmp.mapping(function(fallback)
                    if cmp.visible() then
                        local entry = cmp.get_selected_entry()
                        if entry then
                            vim.api.nvim_put({entry.completion_item.label}, 'c', true, true)
                        end
                        cmp.select_next_item()
                    elseif luasnip.expand_or_jumpable() then
                        luasnip.expand_or_jump()
                    else
                        local success, res = pcall(has_words_before)
                        if success and res then
                            cmp.complete()
                        else
                            fallback()
                        end
                    end
                end, { "i", "s" }),
                ["<S-Tab>"] = cmp.mapping(function(fallback)
                    if cmp.visible() then
                        cmp.select_prev_item()
                    elseif luasnip.jumpable(-1) then
                        luasnip.jump(-1)
                    else
                        fallback()
                    end
                end, { "i", "s" }),
                --      ['<C-b>'] = cmp.mapping.scroll_docs(-4),
                --      ['<C-f>'] = cmp.mapping.scroll_docs(4),
                ['<C-y>'] = cmp.mapping.complete(),
                ['<C-e>'] = cmp.mapping.abort(),
                ['<C-Space>'] = cmp.mapping.confirm({ select = true }),
            }),
            snippet = {
                expand = function(args)
                    luasnip.lsp_expand(args.body)
                end,
            },
            sources = cmp.config.sources({
                { name = 'nvim_lsp' },
                { name = 'luasnip' },
            }, {
                    { name = 'buffer' },
                }),
        })

    end,
}
