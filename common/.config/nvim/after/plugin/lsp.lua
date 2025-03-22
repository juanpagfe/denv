local mason = require("mason");
local masonlsp = require("mason-lspconfig");
local lspconfig = require("lspconfig");
local lsp_defaults = lspconfig.util.default_config;
local cmpnvimlsp = require("cmp_nvim_lsp");


mason.setup()


masonlsp.setup({
  ensure_installed = { "lua_ls", "solargraph" }
})
masonlsp.setup_handlers {
    function (server_name)
        lspconfig[server_name].setup {}
    end,
}

lsp_defaults.capabilities = vim.tbl_deep_extend(
  'force',
  lsp_defaults.capabilities,
  cmpnvimlsp.default_capabilities()
)

lspconfig.pylyzer.setup({
  settings = {
    python = {
      checkOnType = false,   -- Reduce false positives
      diagnostics = true,
    },
  },
})

lspconfig.ts_ls.setup({})

lspconfig.emmet_ls.setup({
    filetypes = { "html", "javascriptreact", "typescriptreact" }
})

local function get_python_path()
  local venv = vim.fn.getenv("VIRTUAL_ENV") -- Get virtual environment path
  if venv and venv ~= vim.NIL then
    return venv .. "/bin/python3" -- Use virtual environment Python
  else
    return vim.fn.exepath("python3") -- Fallback to system Python
  end
end

-- Pyright LSP setup
lspconfig.pyright.setup({
  settings = {
    python = {
      pythonPath = get_python_path(),
    }
  }
})

lspconfig.clangd.setup({
  on_attach = function(client, bufnr)
    -- Enable LSP formatting
    if client.server_capabilities.documentFormattingProvider then
      vim.api.nvim_create_autocmd("BufWritePre", {
        group = vim.api.nvim_create_augroup("Format", { clear = true }),
        buffer = bufnr,
        callback = function() vim.lsp.buf.format() end,
      })
    end
  end,
})



lspconfig.groovyls.setup {
    cmd = { "groovy-language-server" }, -- Or the full path if not in PATH
    filetypes = { "groovy", "Jenkinsfile" },
    root_dir = lspconfig.util.root_pattern("Jenkinsfile", ".git"),
    on_attach = function()
        vim.diagnostic.config({
            virtual_text = {
                prefix = "● ",-- Customize the prefix
            },
            severity_sort = true,
        })
    end,
}

lspconfig.rust_analyzer.setup({
    settings = {
        ["rust-analyzer"] = {
            server = {
                path = "~/.cargo/bin/rust-analyzer"
            };
            cargo = {
                allFeatures = true
            }
        }
    }
})

lspconfig.lua_ls.setup {
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
}

vim.keymap.set('n', '<space>e', vim.diagnostic.open_float)
vim.keymap.set('n', '[d', vim.diagnostic.goto_prev)
vim.keymap.set('n', ']d', vim.diagnostic.goto_next)
vim.keymap.set('n', '<space>q', vim.diagnostic.setloclist)

vim.api.nvim_create_autocmd('LspAttach', {
  group = vim.api.nvim_create_augroup('UserLspConfig', {}),
  callback = function(ev)
    -- Enable completion triggered by <c-x><c-o>
    vim.bo[ev.buf].omnifunc = 'v:lua.vim.lsp.omnifunc'

    -- Buffer local mappings.
    -- See `:help vim.lsp.*` for documentation on any of the below functions
    local opts = { buffer = ev.buf }
    vim.keymap.set('n', 'gD', vim.lsp.buf.declaration, opts)
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', 'gi', vim.lsp.buf.implementation, opts)
    vim.keymap.set('n', '<space>wa', vim.lsp.buf.add_workspace_folder, opts)
    vim.keymap.set('n', '<space>wr', vim.lsp.buf.remove_workspace_folder, opts)
    vim.keymap.set('n', '<space>wl', function()
      print(vim.inspect(vim.lsp.buf.list_workspace_folders()))
    end, opts)
    vim.keymap.set('n', '<space>D', vim.lsp.buf.type_definition, opts)
    vim.keymap.set('n', '<space>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set({ 'n', 'v' }, '<space>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', '<space>f', function()
      vim.lsp.buf.format { async = true }
    end, opts)
  end,
})

vim.diagnostic.config({
    -- update_in_insert = true,
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
