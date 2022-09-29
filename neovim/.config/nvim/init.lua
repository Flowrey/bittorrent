local use = require('packer').use
require('packer').startup(function()
  use 'wbthomason/packer.nvim'
  use 'neovim/nvim-lspconfig'
  use 'rust-lang/rust.vim'
  use 'fatih/vim-go'
  use 'morhetz/gruvbox'
  use { 'nvim-treesitter/nvim-treesitter', run = ':TSUpdate' }
end)

vim.g.gruvbox_italic = 1
vim.g.gruvbox_contrast_dark = "hard"

vim.opt.rnu = true
vim.opt.wildmenu = true
vim.opt.swapfile = false
vim.opt.backup = false
vim.opt.wrap = false
vim.opt.termguicolors = true
vim.opt.background = "dark"

vim.cmd 'colorscheme gruvbox'
vim.cmd 'syntax enable'
vim.cmd 'filetype plugin indent on'

local on_attach = function(client, bufnr)
  vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')
end

require('lspconfig')['pyright'].setup{
    on_attach = on_attach
}
require('lspconfig')['rust_analyzer'].setup{
    on_attach = on_attach
}
require'nvim-treesitter.configs'.setup {
    ensure_installed = { "python", "rust", "go" },
    highlight = {
	enable = true
    }
}
