return {
  "danymat/neogen",
  dependencies = { "L3MON4D3/LuaSnip" },
  event = "VeryLazy",
  config = function()
    local neogen = require("neogen")

    neogen.setup({
      snippet_engine = "luasnip",
    })

    vim.keymap.set("n", "<leader>nf", function()
      neogen.generate({ type = "func" })
    end, { desc = "Generate function doc" })

    vim.keymap.set("n", "<leader>nt", function()
      neogen.generate({ type = "type" })
    end, { desc = "Generate type doc" })
  end,
}
