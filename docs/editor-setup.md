# Editor setup for noyalib-lsp

The noyalib LSP server speaks the standard
[Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
over stdio, so any editor with LSP client support can wire it up.
This page collects the working configurations for the major editors.

For the full list of methods the server implements, see
[`protocol-coverage.md`](./protocol-coverage.md).

## Install

```sh
cargo install noyalib-lsp
```

The binary lands in `~/.cargo/bin/noyalib-lsp`. The remainder of
this document assumes that path is on `PATH`; if not, substitute
the absolute path in each editor's config.

## VS Code

The recommended path is the **noyalib VS Code extension**, which
drives the LSP server and configures it automatically. Install it
from the Marketplace:

```
ext install sebastienrousseau.noyalib
```

Signed release VSIX files are also available from GitHub Releases for manual
installation. The extension starts the binary selected by `noyalib.path`;
install that binary separately and check its version.

For a manual setup (e.g. you want to drive your own LSP client),
add to `settings.json`:

```jsonc
{
  "noyalib.path": "noyalib-lsp",
  "[yaml]": {
    "editor.defaultFormatter": "sebastienrousseau.noyalib",
    "editor.formatOnSave": true
  }
}
```

## Neovim

### nvim-lspconfig

```lua
-- ~/.config/nvim/init.lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

if not configs.noyalib then
  configs.noyalib = {
    default_config = {
      cmd = { "noyalib-lsp" },
      filetypes = { "yaml" },
      root_dir = lspconfig.util.find_git_ancestor,
      single_file_support = true,
    },
  }
end

lspconfig.noyalib.setup({
  on_attach = function(client, bufnr)
    -- Format on save.
    vim.api.nvim_create_autocmd("BufWritePre", {
      buffer = bufnr,
      callback = function()
        vim.lsp.buf.format({ bufnr = bufnr })
      end,
    })
  end,
})
```

### Native vim.lsp (Neovim 0.11+)

```lua
vim.lsp.config.noyalib = {
  cmd = { "noyalib-lsp" },
  filetypes = { "yaml" },
  root_markers = { ".git" },
}
vim.lsp.enable({ "noyalib" })
```

## Helix

`~/.config/helix/languages.toml`:

```toml
[[language]]
name = "yaml"
language-servers = ["noyalib-lsp"]
auto-format = true

[language-server.noyalib-lsp]
command = "noyalib-lsp"
```

## Zed

`~/.config/zed/settings.json`:

```jsonc
{
  "lsp": {
    "noyalib-lsp": {
      "binary": {
        "path": "noyalib-lsp",
        "arguments": []
      }
    }
  },
  "languages": {
    "YAML": {
      "language_servers": ["noyalib-lsp"],
      "format_on_save": "on"
    }
  }
}
```

## Emacs

### lsp-mode

```elisp
;; ~/.emacs.d/init.el
(with-eval-after-load 'lsp-mode
  (add-to-list 'lsp-language-id-configuration '(yaml-mode . "yaml"))
  (lsp-register-client
   (make-lsp-client :new-connection (lsp-stdio-connection "noyalib-lsp")
                    :major-modes '(yaml-mode yaml-ts-mode)
                    :server-id 'noyalib-lsp)))

(add-hook 'yaml-mode-hook #'lsp-deferred)
(add-hook 'before-save-hook #'lsp-format-buffer)
```

### eglot (Emacs 29+)

```elisp
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '((yaml-mode yaml-ts-mode) . ("noyalib-lsp"))))

(add-hook 'yaml-mode-hook 'eglot-ensure)
(add-hook 'yaml-ts-mode-hook 'eglot-ensure)
```

## Sublime Text

Install the LSP package, then add to
`Preferences → Package Settings → LSP → Settings`:

```jsonc
{
  "clients": {
    "noyalib-lsp": {
      "enabled": true,
      "command": ["noyalib-lsp"],
      "selector": "source.yaml"
    }
  }
}
```

## Kakoune

`~/.config/kak/kakrc`:

```kak
hook global WinSetOption filetype=yaml %{
  lsp-enable-window
  set-option global lsp_servers %{
    [noyalib-lsp]
    root_globs = [".git"]
    args = []
  }
}
```

## JetBrains IDEs (IntelliJ, RustRover, GoLand, etc.)

Install the [LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij)
plugin, then in **Settings → LSP**:

- Add a new server named `noyalib-lsp`
- Command: `noyalib-lsp`
- File mappings: `*.yaml`, `*.yml`

## Verifying the server is working

```sh
noyalib-lsp --version
bash examples/handshake.sh
```

Run the handshake from a source checkout after placing the installed binary
on `PATH`. It sends valid Content-Length frames and should return the
initialization capability response and shutdown response. See
[protocol coverage](protocol-coverage.md) for the exact capability object.

## Troubleshooting

### Diagnostics don't appear

Most editors require the LSP client to be configured for the YAML
file type. Confirm with:

```vim
:LspInfo            " Neovim with nvim-lspconfig
:lsp-info           " Helix
M-x lsp-describe-session   " Emacs / lsp-mode
```

### Format-on-save doesn't run

The server provides `textDocument/formatting`. If the editor
doesn't trigger it on save, set the editor's "format on save"
option *and* ensure the LSP client is the registered formatter
for YAML (some editors prefer the built-in tree-sitter formatter
unless told otherwise — see each editor's section above for the
explicit override).

### Schema information is unavailable

Hover reports the overall document type or a parse error. Schema
validation, schema descriptions, completion and document symbols are not
implemented. Editor `yaml.schemas` settings do not enable those features
in this server. Use `noyavalidate` or the core schema API for validation.

### Large buffers use too much CPU

Synchronization replaces and reparses the full buffer. Measure your workload
with the handler benchmarks before making latency claims; see
[`BENCHMARKS.md`](BENCHMARKS.md). The binary's `--help` lists supported
resource options. Report a reproducible slow document with its size and
server version instead of relying on undocumented debounce flags.

## Related

- [Protocol coverage](./protocol-coverage.md) — which LSP methods
  are implemented
- [Crate README](../README.md) — full server overview

## Verify a packaged extension

The installed extension can be tested in an isolated VS Code profile using
the built VSIX and server binary. Set `VS_CODE_CLI` to the editor's command
line launcher. On macOS, set `VS_CODE_BIN` to the application executable
under `Contents/MacOS`.

```sh
node scripts/check-vscode-client.mjs path/to/noyalib.vsix path/to/noyalib-lsp
```

The check installs the VSIX, selects the specified binary, and exercises
diagnostics, Unicode hover, formatting and restart. It prints the editor
version and artifact digests. A local VSIX test does not verify the
marketplace's copy of the extension.
