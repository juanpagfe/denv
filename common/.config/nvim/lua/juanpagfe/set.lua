-- Enable line numbers
vim.opt.nu = true

-- Enable relative line numbers
vim.opt.relativenumber = true

-- Set the number of spaces a tab character represents
vim.opt.tabstop = 4

-- Set the number of spaces a soft tab (backspace key behavior) represents
vim.opt.softtabstop = 4

-- Set the number of spaces to use for auto-indentation
vim.opt.shiftwidth = 4

-- Use spaces instead of tabs when pressing the Tab key
vim.opt.expandtab = true

-- Enable smart indentation based on the code structure
vim.opt.smartindent = true

-- Disable line wrapping (long lines will not wrap)
vim.opt.wrap = true

-- Disable swap file creation
vim.opt.swapfile = false

-- Disable backup file creation
vim.opt.backup = false

-- Highlight the current line under the cursor
vim.opt.cursorline = true

-- Set the undo history directory (use your home directory's `.vim/undodir`)
vim.opt.undodir = os.getenv("HOME") .. "/.vim/undodir"

-- Enable undo file persistence
vim.opt.undofile = true

-- Disable highlighting of search results after the search is completed
vim.opt.hlsearch = false

-- Enable incremental search (search results update as you type)
vim.opt.incsearch = true

-- Enable 24-bit color support (requires terminal support)
vim.opt.termguicolors = true

-- Set the number of lines to keep above and below the cursor when scrolling
vim.opt.scrolloff = 8

-- Always show the sign column (used for diagnostics, git signs, etc.)
vim.opt.signcolumn = "yes"

-- Append "@" to the list of characters considered part of a filename
vim.opt.isfname:append("@-@")

-- Set the update time for events like auto-write and cursor updates (in milliseconds)
vim.opt.updatetime = 50

-- Highlight the 80th column as a visual guide (useful for line length limits)
vim.opt.colorcolumn = "80"

-- Set the leader key to the spacebar for mappings
vim.g.mapleader = " "

vim.g.netrw_banner = 0

vim.opt.guicursor = "n-v-c:block,i-ci-ve:ver25,r-cr:hor20,o:hor50"
