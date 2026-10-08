# noyalib YAML for Visual Studio Code

Diagnostics as you type, formatting on save and JSON Schema descriptions on
hover, from the [noyalib-lsp](https://github.com/sebastienrousseau/noyalib-lsp)
language server.

## Requirements

The server binary:

```bash
cargo install noyalib-lsp --locked
```

or a signed binary from the
[releases](https://github.com/sebastienrousseau/noyalib-lsp/releases). Set
`noyalib.path` if it is not on your PATH.

## Settings

| Setting | Default | Meaning |
|---|---|---|
| `noyalib.path` | `noyalib-lsp` | The server binary to run: a command on PATH or an absolute path |
| `noyalib.trace.server` | `off` | Protocol tracing |

The server starts in your home directory. A relative `noyalib.path` is
refused, and in a workspace you have not trusted the workspace's
`noyalib.path` is ignored (VS Code restricted mode), so opening a folder
cannot make the extension run a program from it.

To format on save:

```json
{ "[yaml]": { "editor.defaultFormatter": "sebastienrousseau.noyalib", "editor.formatOnSave": true } }
```

## Build the extension

```bash
cd editors/vscode && npm install && npm run package   # noyalib-<version>.vsix
```

Install the `.vsix` with "Extensions: Install from VSIX…". Each release
attaches the packaged extension.
