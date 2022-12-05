require'nvim-treesitter.configs'.setup {
	ensure_installed = { "python", "rust", "go", "lua", "json", "toml", "yaml", "elixir" },
	highlight = {
		enable = true
	}
}
