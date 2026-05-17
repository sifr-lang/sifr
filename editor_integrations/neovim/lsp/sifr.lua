return {
  cmd = { "sifr", "lsp", "--stdio" },
  filetypes = { "sifr" },
  root_markers = { "sifr.toml", ".git" },
  settings = {
    sifr = {
      diagnostics = {
        mode = "open-files",
      },
      format = {
        enable = true,
      },
      lint = {
        enable = true,
      },
    },
  },
}
