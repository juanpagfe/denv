return {
  "lewis6991/gitsigns.nvim",
  event = "BufReadPre",
  config = function()
    require("gitsigns").setup {
      on_attach = function(bufnr)
        local function map(mode, lhs, rhs, opts)
          opts = vim.tbl_extend("force", { noremap = true, silent = true }, opts or {})
          vim.keymap.set(mode, lhs, rhs, opts)
        end

        -- Navigation
        map("n", "]c", "&diff ? ']c' : '<cmd>Gitsigns next_hunk<CR>'", { expr = true, buffer = bufnr })
        map("n", "[c", "&diff ? '[c' : '<cmd>Gitsigns prev_hunk<CR>'", { expr = true, buffer = bufnr })

        -- Actions
        map("n", "<leader>hs", ":Gitsigns stage_hunk<CR>", { buffer = bufnr })
        map("v", "<leader>hs", ":Gitsigns stage_hunk<CR>", { buffer = bufnr })
        map("n", "<leader>hr", ":Gitsigns reset_hunk<CR>", { buffer = bufnr })
        map("v", "<leader>hr", ":Gitsigns reset_hunk<CR>", { buffer = bufnr })
        map("n", "<leader>hS", "<cmd>Gitsigns stage_buffer<CR>", { buffer = bufnr })
        map("n", "<leader>hu", "<cmd>Gitsigns undo_stage_hunk<CR>", { buffer = bufnr })
        map("n", "<leader>hR", "<cmd>Gitsigns reset_buffer<CR>", { buffer = bufnr })
        map("n", "<leader>hp", "<cmd>Gitsigns preview_hunk<CR>", { buffer = bufnr })
        map("n", "<leader>hb", function() require("gitsigns").blame_line { full = true } end, { buffer = bufnr })
        map("n", "<leader>tb", "<cmd>Gitsigns toggle_current_line_blame<CR>", { buffer = bufnr })
        map("n", "<leader>hd", "<cmd>Gitsigns diffthis<CR>", { buffer = bufnr })
        map("n", "<leader>hD", function() require("gitsigns").diffthis("~") end, { buffer = bufnr })
        map("n", "<leader>td", "<cmd>Gitsigns toggle_deleted<CR>", { buffer = bufnr })

        -- Text object
        map("o", "ih", ":<C-U>Gitsigns select_hunk<CR>", { buffer = bufnr })
        map("x", "ih", ":<C-U>Gitsigns select_hunk<CR>", { buffer = bufnr })
      end
    }
  end
}
