" ============================================================================
" VIM Configuration
" Generated from nvim settings
" ============================================================================

" Set the leader key to spacebar
let mapleader = " "

" ============================================================================
" Display Settings
" ============================================================================

" Enable line numbers
set number

" Enable relative line numbers
set relativenumber

" Highlight the current line under the cursor
set cursorline

" Highlight the 80th column as a visual guide
set colorcolumn=80

" Always show the sign column
set signcolumn=yes

" Set cursor shape for different modes
set guicursor=n-v-c:block,i-ci-ve:ver25,r-cr:hor20,o:hor50

" Enable 24-bit color support (requires terminal support)
set termguicolors

" ============================================================================
" Indentation and Formatting
" ============================================================================

" Set the number of spaces a tab character represents
set tabstop=4

" Set the number of spaces a soft tab (backspace key behavior) represents
set softtabstop=4

" Set the number of spaces to use for auto-indentation
set shiftwidth=4

" Use spaces instead of tabs when pressing the Tab key
set expandtab

" Enable smart indentation based on the code structure
set smartindent

" Enable line wrapping
set wrap

" ============================================================================
" Search Settings
" ============================================================================

" Disable highlighting of search results after the search is completed
set nohlsearch

" Enable incremental search (search results update as you type)
set incsearch

" ============================================================================
" File and Backup Settings
" ============================================================================

" Disable swap file creation
set noswapfile

" Disable backup file creation
set nobackup

" Set the undo history directory
set undodir=~/.vim/undodir

" Enable undo file persistence
set undofile

" Auto-reload files when changed outside of Vim
set autoread

" ============================================================================
" Scrolling and Navigation
" ============================================================================

" Set the number of lines to keep above and below the cursor when scrolling
set scrolloff=8

" ============================================================================
" Performance Settings
" ============================================================================

" Set the update time for events like auto-write and cursor updates (in milliseconds)
set updatetime=50

" Enable lazy redraw for better performance
set lazyredraw

" ============================================================================
" Netrw Settings
" ============================================================================

" Disable netrw banner
let g:netrw_banner = 0

" ============================================================================
" File Name Settings
" ============================================================================

" Append "@" to the list of characters considered part of a filename
set isfname+=@-@

" ============================================================================
" Key Mappings
" ============================================================================

" Open directory explorer
nnoremap <leader><Esc> :Ex<CR>

" Move selected lines down in visual mode
vnoremap J :m '>+1<CR>gv=gv

" Move selected lines up in visual mode
vnoremap K :m '<-2<CR>gv=gv

" Combine current line with next line, preserving cursor position
nnoremap J mzJ`z

" Scroll down half page and center cursor
nnoremap <C-d> <C-d>zz

" Scroll up half page and center cursor
nnoremap <C-u> <C-u>zz

" Search next match and center on screen
nnoremap n nzzzv

" Search previous match and center on screen
nnoremap N Nzzzv

" Paste without affecting the unnamed register
xnoremap <leader>p "_dP

" Yank to system clipboard
nnoremap <leader>y "+y
vnoremap <leader>y "+y

" Yank entire line to system clipboard
nnoremap <leader>Y "+Y

" Delete without affecting the unnamed register
nnoremap <leader>d "_d
vnoremap <leader>d "_d

" Disable Q key
nnoremap Q <nop>

" Jump to next quickfix item and center on screen
nnoremap <C-k> :cnext<CR>zz

" Jump to previous quickfix item and center on screen
nnoremap <C-j> :cprev<CR>zz

" Jump to next location list item and center on screen
nnoremap <leader>k :lnext<CR>zz

" Jump to previous location list item and center on screen
nnoremap <leader>j :lprev<CR>zz

" Search and replace current word under cursor
nnoremap <leader>s :%s/\<<C-r><C-w>\>/<C-r><C-w>/gI<Left><Left><Left>

" Start search for current word under cursor
nnoremap <leader>f /<C-r><C-w><Left><Left><Left>

" Make current file executable
nnoremap <leader>x :!chmod +x %<CR>

" Switch to next buffer
nnoremap <Tab> :bnext<CR>

" Switch to previous buffer
nnoremap <S-Tab> :bprevious<CR>

" Delete current buffer (note: conflicts with delete mapping above)
" nnoremap <leader>d :bd<CR>

" ============================================================================
" Auto Commands
" ============================================================================

" Auto-reload files when focus is gained or buffer is entered
augroup AutoReload
    autocmd!
    autocmd FocusGained,BufEnter * checktime
augroup END

" ============================================================================
" Custom Commands
" ============================================================================

" Format JSON using Python
command! Json %!python3 -m json.tool

" ============================================================================
" End of Configuration
" ============================================================================
