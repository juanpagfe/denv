return {
    "neovim/nvim-lspconfig",
    dependencies = {
        "stevearc/conform.nvim",
        "williamboman/mason.nvim",
        "williamboman/mason-lspconfig.nvim",
        "hrsh7th/cmp-nvim-lsp",
        "hrsh7th/cmp-buffer",
        "hrsh7th/cmp-path",
        "hrsh7th/cmp-cmdline",
        "hrsh7th/nvim-cmp",
        "L3MON4D3/LuaSnip",
        "saadparwaiz1/cmp_luasnip"
    },
    config = function()
        require("conform").setup({
            formatters_by_ft = {
            }
        })
        local mason = require("mason")
        local masonlsp = require("mason-lspconfig")
        local lspconfig = require("lspconfig")
        local cmp_nvim_lsp = require("cmp_nvim_lsp")

        mason.setup()

        -- Update LSP capabilities for completion
        local capabilities = vim.tbl_deep_extend(
            "force",
            lspconfig.util.default_config.capabilities,
            cmp_nvim_lsp.default_capabilities()
        )

        masonlsp.setup({
            ensure_installed = { "lua_ls", "solargraph" },
            handlers = {
                function(server_name) -- default handler (optional)
                    require("lspconfig")[server_name].setup {
                        capabilities = capabilities
                    }
                end,
                ["lua_ls"] = function()
                    lspconfig.lua_ls.setup {
                        capabilities = capabilities,
                        settings = {
                            Lua = {
                                runtime = { version = "Lua 5.1" },
                                diagnostics = {
                                    globals = { "bit", "vim", "it", "describe", "before_each", "after_each" },
                                }
                            }
                        }
                    }
                end,
            }
        })

        -- Specific LSP configurations
        lspconfig.pylyzer.setup({
            settings = {
                python = {
                    checkOnType = false,
                    diagnostics = true,
                },
            },
        })

        lspconfig.ts_sl.setup({})
        lspconfig.emmet_ls.setup({
            filetypes = { "html", "javascriptreact", "typescriptreact" }
        })

        local function get_python_path()
            local venv = vim.fn.getenv("VIRTUAL_ENV")
            if venv and venv ~= vim.NIL then
                return venv .. "/bin/python3"
            else
                return vim.fn.exepath("python3")
            end
        end

        lspconfig.pyright.setup({
            settings = {
                python = {
                    pythonPath = get_python_path(),
                }
            }
        })

        lspconfig.clangd.setup({
            on_attach = function(client, bufnr)
                if client.server_capabilities.documentFormattingProvider then
                    vim.api.nvim_create_autocmd("BufWritePre", {
                        group = vim.api.nvim_create_augroup("Format", { clear = true }),
                        buffer = bufnr,
                        callback = function() vim.lsp.buf.format() end,
                    })
                end
            end,
        })

        lspconfig.groovyls.setup({
            cmd = { "groovy-language-server" },
            filetypes = { "groovy", "Jenkinsfile" },
            root_dir = lspconfig.util.root_pattern("Jenkinsfile", ".git"),
            on_attach = function()
                vim.diagnostic.config({
                    virtual_text = { prefix = "● " },
                    severity_sort = true,
                })
            end,
        })

        lspconfig.rust_analyzer.setup({
            settings = {
                ["rust-analyzer"] = {
                    server = { path = "~/.cargo/bin/rust-analyzer" },
                    cargo = { allFeatures = true },
                }
            }
        })

        lspconfig.lua_ls.setup({
            settings = {
                Lua = {
                    diagnostics = {
                        globals = { "vim" },
                    },
                    workspace = {
                        library = {
                            [vim.fn.expand "$VIMRUNTIME/lua"] = true,
                            [vim.fn.stdpath "config" .. "/lua"] = true,
                        },
                    },
                },
            }
        })

        vim.api.nvim_create_autocmd("LspAttach", {
            group = vim.api.nvim_create_augroup("UserLspConfig", {}),
            callback = function(ev)
                local opts = { buffer = ev.buf }
                vim.bo[ev.buf].omnifunc = "v:lua.vim.lsp.omnifunc"
                vim.keymap.set("n", "gD", vim.lsp.buf.declaration, opts)
                vim.keymap.set("n", "gd", vim.lsp.buf.definition, opts)
                vim.keymap.set("n", "K", vim.lsp.buf.hover, opts)
                vim.keymap.set("n", "gi", vim.lsp.buf.implementation, opts)
                vim.keymap.set("n", "<space>wa", vim.lsp.buf.add_workspace_folder, opts)
                vim.keymap.set("n", "<space>wr", vim.lsp.buf.remove_workspace_folder, opts)
                vim.keymap.set("n", "<space>wl", function()
                    print(vim.inspect(vim.lsp.buf.list_workspace_folders()))
                end, opts)
                vim.keymap.set("n", "<space>D", vim.lsp.buf.type_definition, opts)
                vim.keymap.set("n", "<space>rn", vim.lsp.buf.rename, opts)
                vim.keymap.set({ "n", "v" }, "<space>ca", vim.lsp.buf.code_action, opts)
                vim.keymap.set("n", "gr", vim.lsp.buf.references, opts)
                vim.keymap.set("n", "<space>f", function()
                    vim.lsp.buf.format { async = true }
                end, opts)
            end,
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

        lspconfig.eslint.setup({
            on_attach = function(client, bufnr)
                vim.api.nvim_create_autocmd("BufWritePre", {
                    buffer = bufnr,
                    command = "EslintFixAll",
                })
            end,
        })
        local cmp = require("cmp")
        local luasnip = require("luasnip")

        require("luasnip.loaders.from_vscode").lazy_load()

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

