filetype plugin indent on
syntax on

let mapleader = ","

let g:gruvbox_italic = 1
let g:gruvbox_contrast_dark = "hard"
" let g:ale_completion_enabled = 0
" let g:ale_completion_autoimport = 0
let g:ale_linters_explicit = 1
" let g:ale_linters = {
" \   'python': ['pyright'],
" \}


if !has('gui_running') && &term =~ '^\%(screen\|tmux\)'
  let &t_8f = "\<Esc>[38;2;%lu;%lu;%lum"
  let &t_8b = "\<Esc>[48;2;%lu;%lu;%lum"
endif

set omnifunc=ale#completion#OmniFunc
set termguicolors

set background=dark
set mouse=a

set number
set wildmenu
set relativenumber
set autowrite
set hidden
set noswapfile
set nobackup
set nowrap

autocmd vimenter * ++nested colorscheme gruvbox

" Plugins:
" gruvbox: https://github.com/morhetz/gruvbox 
" vim-fugitive: https://github.com/tpope/vim-fugitive 
" vim-go: https://github.com/fatih/vim-go
" ctrlp: https://github.com/kien/ctrlp.vim.git
" vim-commentary: https://github.com/tpope/vim-commentary.git
" vim-surround: https://github.com/tpope/vim-surround.git
" syntastic: https://github.com/vim-syntastic/syntastic 
" ale: https://github.com/dense-analysis/ale
