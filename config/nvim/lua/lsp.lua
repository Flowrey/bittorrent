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
				typeCheckingMode = "off",
				useLibraryCodeForTypes = true
			}
		}
	}
}

require('lspconfig')['rust_analyzer'].setup{
	on_attach = on_attach
}

