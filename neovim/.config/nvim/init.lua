local use = require('packer').use
require('packer').startup(function()
    use 'wbthomason/packer.nvim'
    use 'neovim/nvim-lspconfig'
    use 'rust-lang/rust.vim'
    use 'fatih/vim-go'
    use 'morhetz/gruvbox'
    use 'tpope/vim-commentary'
    use 'tpope/vim-surround'
    use 'tpope/vim-fugitive'
    use { 'nvim-treesitter/nvim-treesitter', run = ':TSUpdate' }
    use { 'nvim-telescope/telescope.nvim', tag = '0.1.0', requires = { {'nvim-lua/plenary.nvim'} } }
end)

vim.g.gruvbox_italic = 1
vim.g.gruvbox_contrast_dark = "hard"

vim.opt.rnu = true
vim.opt.nu = true
vim.opt.tabstop = 4
vim.opt.softtabstop = 4
vim.opt.shiftwidth = 4
vim.opt.expandtab = true
vim.opt.autoindent = true
vim.opt.smartindent = true
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
	on_attach = on_attach,
	settings = {
		python = {
			analysis = {
				autoSearchPaths = true,
				diagnosticMode = "workspace",
				typeCheckingMode = "strict",
				useLibraryCodeForTypes = true
			}
		}
	}
}

require('lspconfig')['rust_analyzer'].setup{
	on_attach = on_attach
}
require'nvim-treesitter.configs'.setup {
	ensure_installed = { "python", "rust", "go", "lua" },
	highlight = {
		enable = true
	}
}
