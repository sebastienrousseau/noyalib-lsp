<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Architecture

`noyalib-lsp` is a Language Server Protocol implementation for YAML,
built on noyalib's parser, CST formatter, and error locations. The
transport is stdio with the standard `Content-Length` framing.

## Layers

- **`src/main.rs`** is the transport shim: it frames messages and
  drives `Server::handle_message`. All behaviour lives in the library
  so `cargo test` covers it without an editor.
- **`src/lib.rs`** holds `Server`, which owns the document store of
  open buffers, and `handle_message`, which parses the JSON-RPC
  envelope, routes by method, and returns a `HandleOutcome` of one of
  three kinds: `reply` (a response), `notify` (a server-initiated
  notification such as diagnostics), or `silent`. The capability set
  advertised at `initialize` is fixed: `textDocumentSync` (full),
  `textDocument/formatting`, and `textDocument/hover`, with
  `textDocument/publishDiagnostics` pushed after each change.
- **`src/diagnostics.rs`** turns noyalib parse errors into LSP
  diagnostic objects (`collect`) and wraps them in the
  `publishDiagnostics` notification (`publish_diagnostics`).
- **`src/format.rs`** implements formatting as a single `TextEdit`
  replacing the whole document with noyalib's CST formatter output
  (`full_document_edits`). One edit keeps the response self-contained;
  the client needs no cross-document reasoning to apply it.
- **`src/hover.rs`** answers `textDocument/hover` (`hover_at`): a
  small Markdown card with the position and the document's overall
  type when the buffer parses, or the parse error at the cursor when it
  does not. `byte_offset_of` maps an LSP line and column to a byte
  offset. Schema-driven hover descriptions are tracked as future work.

## Data flow

Editor sends `didOpen` or `didChange` with the full text; the server
stores it, parses it, and pushes diagnostics. Formatting and hover read
the stored text on request. Nothing is written to disk.

## Testing

Unit tests drive `Server::handle_message` with raw JSON-RPC strings.
Two libFuzzer targets feed it arbitrary messages (`fuzz_handle_message`)
and arbitrary document text through `didChange` (`fuzz_did_change`);
CI replays the seed corpus on every push. `docs/protocol-coverage.md`
lists which LSP methods are implemented.

## Lockstep

The crate pins `noyalib` at the identical `=0.0.X` and releases with
it (core ADR-0005).
