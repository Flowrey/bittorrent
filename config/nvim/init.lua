require('plugins')
require('treesitter')
require('lsp')
require('completion')

vim.g.mapleader = " "
vim.g.gruvbox_italic = 1
vim.g.gruvbox_contrast_dark = "medium"

vim.keymap.set("n", "<Leader><Space>", ":noh<cr>", {noremap=true})

vim.keymap.set("t", "<Esc>", "<C-\\><C-n>", {noremap = true})
vim.keymap.set("t", "<A-h>", "<C-\\><C-N><C-w>h", {noremap = true})
vim.keymap.set("t", "<A-j>", "<C-\\><C-N><C-w>j", {noremap = true})
vim.keymap.set("t", "<A-k>", "<C-\\><C-N><C-w>k", {noremap = true})
vim.keymap.set("t", "<A-l>", "<C-\\><C-N><C-w>l", {noremap = true})
vim.keymap.set("i", "<A-h>", "<C-\\><C-N><C-w>h", {noremap = true})
vim.keymap.set("i", "<A-j>", "<C-\\><C-N><C-w>j", {noremap = true})
vim.keymap.set("i", "<A-k>", "<C-\\><C-N><C-w>k", {noremap = true})
vim.keymap.set("i", "<A-l>", "<C-\\><C-N><C-w>l", {noremap = true})
vim.keymap.set("n", "<A-h>", "<C-w>h", {noremap = true})
vim.keymap.set("n", "<A-j>", "<C-w>j", {noremap = true})
vim.keymap.set("n", "<A-k>", "<C-w>k", {noremap = true})
vim.keymap.set("n", "<A-l>", "<C-w>l", {noremap = true})

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
vim.cmd 'autocmd TermOpen * setlocal nonumber norelativenumber'

