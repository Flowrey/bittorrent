filetype plugin indent on
syntax on

let mapleader = ","

let g:gruvbox_italic=1
let g:gruvbox_termcolors=16

let g:syntastic_always_populate_loc_list = 1
let g:syntastic_auto_loc_list = 0
let g:syntastic_check_on_open = 1
let g:syntastic_check_on_wq = 0
let g:syntastic_python_checkers = ["flake8"]
let g:syntastic_python_flake8_args='--ignore=E501'
let g:syntastic_mode_map = {
	\ "mode": "active",
	\ "passive_filetypes": ["go"] }

set background=dark

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
