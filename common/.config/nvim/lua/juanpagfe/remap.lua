-- Opens the directory above the current buffer (using Ex command)
vim.keymap.set("n", "<leader><Esc>", vim.cmd.Ex)

-- In visual mode, moves the current line down by one position
vim.keymap.set("v", "J", ":m '>+1<CR>gv=gv")

-- In visual mode, moves the current line up by one position
vim.keymap.set("v", "K", ":m '<-2<CR>gv=gv")

-- In normal mode, combines the current line with the next line, preserving the cursor position
vim.keymap.set("n", "J", "mzJ`z")

-- In normal mode, scrolls the page down while keeping the cursor centered
vim.keymap.set("n", "<C-d>", "<C-d>zz")

-- In normal mode, scrolls the page up while keeping the cursor centered
vim.keymap.set("n", "<C-u>", "<C-u>zz")

-- In normal mode, performs a search for the next match and centers it on the screen
vim.keymap.set("n", "n", "nzzzv")

-- In normal mode, performs a search for the previous match and centers it on the screen
vim.keymap.set("n", "N", "Nzzzv")

-- In visual mode, pastes without affecting the unnamed register
vim.keymap.set("x", "<leader>p", [["_dP]])

-- In normal and visual modes, yank to the system clipboard (copy text)
vim.keymap.set({ "n", "v" }, "<leader>y", [["+y]])

-- In normal mode, yanks the entire line to the system clipboard
vim.keymap.set("n", "<leader>Y", [["+Y]])

-- In normal and visual modes, delete without affecting the unnamed register
vim.keymap.set({ "n", "v" }, "<leader>d", [["_d]])

-- Disabled: Mapping <C-c> to escape in insert mode, comment is left for clarity
-- vim.keymap.set("i", "<C-c>", "<Esc>")

-- In normal mode, disables the 'Q' key (remaps it to do nothing)
vim.keymap.set("n", "Q", "<nop>")

-- In normal mode, opens a new tmux window for a terminal session
vim.keymap.set("n", "<C-f>", "<cmd>silent !tmux neww tsess<CR>")

-- In normal mode, formats the current buffer using LSP
vim.keymap.set("n", "<leader>fa", vim.lsp.buf.format)

-- In normal mode, jumps to the next quickfix item and centers it on the screen
vim.keymap.set("n", "<C-k>", "<cmd>cnext<CR>zz")

-- In normal mode, jumps to the previous quickfix item and centers it on the screen
vim.keymap.set("n", "<C-j>", "<cmd>cprev<CR>zz")

-- In normal mode, jumps to the next location list item and centers it on the screen
vim.keymap.set("n", "<leader>k", "<cmd>lnext<CR>zz")

-- In normal mode, jumps to the previous location list item and centers it on the screen
vim.keymap.set("n", "<leader>j", "<cmd>lprev<CR>zz")

-- In normal mode, replaces the current word under the cursor with itself (for quick search/replace)
vim.keymap.set("n", "<leader>s", [[:%s/\<<C-r><C-w>\>/<C-r><C-w>/gI<Left><Left><Left>]])

-- In normal mode, starts a search for the current word under the cursor
vim.keymap.set("n", "<leader>f", [[/<C-r><C-w><Left><Left><Left>]])

-- In normal mode, makes the current file executable (if it's a script)
vim.keymap.set("n", "<leader>x", "<cmd>!chmod +x %<CR>", { silent = true })

-- In normal mode, jumps to the previous selected buffer
vim.api.nvim_set_keymap('n', '<leader><Tab>', ":lua require('telescope.builtin').buffers { sort_lastused = true, ignore_current_buffer = true, only_cwd = true }<CR>", { noremap = true, silent = true })

-- In visual mode, replaces the selected text with a substitution pattern (using the selected text as part of the command)
vim.keymap.set('v', '<leader>p', function()
  -- Get the selected text
  local selected_text = vim.fn.getreg('"')

  vim.cmd('normal! <Esc>')  -- Exit visual mode

  -- Construct the substitution command
  local command = string.format(':%%s/<%s>/Hello World/g', selected_text)

  -- Execute the command
  vim.api.nvim_input(command)
end, { noremap = true, silent = true })

-- Disabled: Commented out mapping to use double <leader> for sourcing configuration
-- vim.keymap.set("n", "<leader><leader>", function()
--     vim.cmd("so")
-- end)

