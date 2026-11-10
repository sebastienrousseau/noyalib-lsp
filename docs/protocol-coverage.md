<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# LSP protocol coverage

The server implements the following subset of LSP 3.17 over
`Content-Length`-framed stdio. The source of truth is `src/lib.rs::dispatch`;
`tests/protocol.rs` exercises the actual binary and `tests/framing.rs`
exercises the framing boundary.

## Implemented messages

| Method | Direction | Result |
|---|---|---|
| `initialize` | Request | Formatting, hover and full-document sync capabilities; package version |
| `initialized` | Notification | Accepted |
| `shutdown` | Request | Begins shutdown; later requests are refused |
| `exit` | Notification | Ends the process; lifecycle exit status is enforced by the binary |
| `textDocument/didOpen` | Notification | Stores the buffer and publishes parse diagnostics |
| `textDocument/didChange` | Notification | Replaces the whole buffer and republishes parse diagnostics; stale or duplicate versions are ignored |
| `textDocument/didClose` | Notification | Removes the tracked buffer |
| `textDocument/formatting` | Request | CST-backed full-document edits |
| `textDocument/hover` | Request | Document type, cursor position or parse error; not schema descriptions |
| `textDocument/publishDiagnostics` | Server notification | Parse diagnostics, including errors in later stream documents; an empty list clears stale errors |

Before `initialize`, requests receive `-32002` and notifications are ignored
except `exit`. An exit after a successful shutdown returns code 0; exit
without shutdown returns code 1, without waiting for the client to close
stdin. The framer refuses oversized headers even when their terminator
arrives in the same read.

Unknown requests return JSON-RPC `-32601` (method not found). Unknown
notifications have no response, as required by JSON-RPC. `didSave`,
`didChangeConfiguration` and `didChangeWatchedFiles` have no handlers;
they are not implemented configuration or file-watching features.

## Unsupported capabilities

Completion, symbols, code actions, rename, references, definitions, folding,
workspace symbols, schema validation and schema-aware hover are unsupported.
The server neither downloads a schema catalogue nor consumes `yaml.schemas`
or `$schema` pragmas. Use the core schema API or `noyavalidate` for schema
validation. These limits apply even if another editor extension offers the
same capabilities independently.

## Initialization contract

The capability object is:

```json
{
  "textDocumentSync": 1,
  "documentFormattingProvider": true,
  "hoverProvider": true
}
```

`1` means full-document synchronization. `serverInfo.version` comes from
`CARGO_PKG_VERSION`; it is not a separately hard-coded protocol version.
The server does not advertise a pull-diagnostic provider or completion.
Diagnostics use zero-based UTF-16 positions. Hover coordinates follow the
same protocol encoding. The server reparses full buffers; performance claims
require measured results in [BENCHMARKS.md](BENCHMARKS.md).

## Exercising the contract

```sh
cargo test --test protocol --test framing
bash examples/handshake.sh
```

The handshake example requires an installed `noyalib-lsp` on `PATH`; it
frames initialization, shutdown and exit correctly. Raw newline-delimited
JSON is not an LSP transport.

See [editor setup](editor-setup.md) for client configuration and
[the README](../README.md) for the supported scope.
