<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/noyalib/v1/logos/noyalib.svg" alt="noyalib-lsp logo" width="128" />
</p>

<h1 align="center">noyalib-lsp</h1>

<p align="center">
  A YAML language server providing formatting, diagnostics, document symbols, and schema-aware hover over standard LSP.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/noyalib-lsp/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/noyalib-lsp/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/noyalib-lsp"><img src="https://img.shields.io/crates/v/noyalib-lsp.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Registry" /></a>
  <a href="https://docs.rs/noyalib-lsp"><img src="https://img.shields.io/badge/docs.rs-noyalib--lsp-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/noyalib-lsp"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/noyalib-lsp?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
  <a href="LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg?style=for-the-badge" alt="License: Apache-2.0 OR MIT" /></a>
  <a href="https://github.com/sebastienrousseau/noyalib-lsp/blob/main/docs/POLICIES.md"><img src="https://img.shields.io/badge/MSRV-1.86.0-93450a.svg?style=for-the-badge&logo=rust" alt="MSRV 1.86.0" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install) — Cargo and source builds
- [Requirements](#requirements) — toolchain floor, platforms
- [Quick Start](#quick-start) — connect an editor to the server

**The noyalib-lsp ecosystem**

- [The noyalib-lsp ecosystem](#the-noyalib-lsp-ecosystem) — server, extension, and core library

**Library reference**

- [Capabilities at a glance](#capabilities-at-a-glance) — the current surface by theme
- [Ecosystem comparison](#ecosystem-comparison) — short matrix; full table at [`docs/COMPARISON.md`](docs/COMPARISON.md)
- [Benchmarks](#benchmarks) — headline numbers; full table at [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md)
- [Features](#features) — module-level capability list
- [Configuration](#configuration) — core options
- [Examples](#examples) — runnable example index

**Operational**

- [When not to use noyalib-lsp](#when-not-to-use-noyalib-lsp) — limitations
- [Development](#development) — make targets, fuzzing, CI
- [Security](#security) — guarantees and compliance
- [Documentation](#documentation) — all reference docs
- [Stability guarantees](#stability-guarantees) — SemVer axis, output stability, minimum toolchain discipline
- [License](#license)

---

## Install

### As a Rust library

```toml
[dependencies]
noyalib-lsp = "0.0.46"
```

Install the language-server binary with Cargo:

```bash
cargo install noyalib-lsp --locked
```

The VS Code extension source lives under [`editors/vscode/`](editors/vscode/).
Other LSP clients start the `noyalib-lsp` binary over stdio.

---

## Requirements

- Rust **1.86.0 or newer** when building from source.
- Linux, macOS, and Windows are tested on stable, beta, and nightly Rust.
- An editor or client supporting LSP 3.17 over `Content-Length` framed stdio.
- The server pins `noyalib` at exactly `=0.0.46`.

| Surface | Minimum toolchain | Enforcement |
| :--- | :---: | :--- |
| Server and library | Rust 1.86.0 | manifest and MSRV CI |
| VS Code extension | Declared Node runtime | extension build CI |

---

## Quick Start

```bash
noyalib-lsp --version
```

The first framed client request is a standard LSP initialization message:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "processId": null,
    "rootUri": null,
    "capabilities": {}
  }
}
```

Configure an editor to launch `noyalib-lsp` for YAML buffers. The complete
configurations for VS Code, Zed, Neovim, Helix, Emacs, and Sublime Text live in
[`docs/editor-setup.md`](docs/editor-setup.md).

---

## The noyalib-lsp ecosystem

The server is the editor-facing member of the lockstep noyalib family.

| Component | Purpose | Use case |
| :--- | :--- | :--- |
| `noyalib-lsp` | Standard LSP server | Any LSP-aware editor |
| `editors/vscode` | VS Code extension | Packaged VS Code integration |
| [`noyalib`](https://github.com/sebastienrousseau/noyalib) | Parser, CST, and schema engine | Formatting and diagnostics |
| [`noya-cli`](https://github.com/sebastienrousseau/noya-cli) | Batch formatter and validator | Shell and CI workflows |

---

## Capabilities at a glance

| Area | Capability | Status |
| :--- | :--- | :--- |
| Synchronisation | Open, change, save, and close documents | Stable |
| Diagnostics | YAML parse and schema diagnostics | Stable |
| Formatting | Full-document CST-backed edits | Stable |
| Navigation | Document symbols | Stable |
| Assistance | Hover and completion from JSON Schema | Stable |
| Transport | Standard stdio LSP framing | Stable |

---

## Ecosystem comparison

`noyalib-lsp` prioritises noyalib's lossless formatter and parser semantics. It
does not bundle a remote schema catalogue.

| Project | Lossless formatter | Schema hover | Bundled catalogue |
| :--- | :---: | :---: | :---: |
| **noyalib-lsp** | Yes | Yes | No |
| `yaml-language-server` | Reprints | Yes | Yes |
| Generic formatter integration | Tool-dependent | No | No |

See [`docs/COMPARISON.md`](docs/COMPARISON.md) for the evidence and complete matrix.

---

## Benchmarks

The `lsp_handlers` Criterion harness measures request dispatch and formatting.
CI smoke-runs the benchmark target to prevent silent API or build drift.

| Scenario | Result | Environment |
| :--- | ---: | :--- |
| Handler dispatch | Measured by `lsp_handlers` | Criterion release build |
| Formatting | Dominated by changed-buffer parsing | Native release build |
| Benchmark compilation | Per push | CI smoke gate |

See [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) for methodology and full results.

---

## Features

- Publish diagnostics after document changes.
- Format documents through noyalib's comment-preserving CST.
- Return schema-backed hover and completion data.
- Enumerate document symbols for mappings and sequences.
- Run in VS Code, Zed, Neovim, Helix, Emacs, and Sublime Text.
- Expose a Rust library for protocol and handler integration tests.

---

## Configuration

The server consumes editor-provided initialization options and schema mappings.
Editor-specific JSON and Lua examples are maintained in
[`docs/editor-setup.md`](docs/editor-setup.md). Protocol coverage is tracked in
[`docs/protocol-coverage.md`](docs/protocol-coverage.md).

---

## Examples

- [`handshake.sh`](examples/handshake.sh): raw LSP initialization.
- [`format-on-save.sh`](examples/format-on-save.sh): formatting request flow.
- [`hover-cursor.sh`](examples/hover-cursor.sh): schema-aware hover.

The examples speak framed JSON-RPC directly and can be run against a local
debug or release binary.

---

## When not to use noyalib-lsp

- Use `noya-cli` for batch CI validation without an editor process.
- Choose a server with a bundled schema registry if automatic Kubernetes or
  OpenAPI schema discovery is mandatory.
- Choose another implementation when cross-file refactors and workspace edits
  are required today.

The [detailed README reference](docs/README-REFERENCE.md) retains full editor
configuration and protocol rationale.

---

## Development

```bash
make
make test
make clippy
make fmt
make doc
```

CI enforces the OS and toolchain matrix, MSRV, strict rustdoc, coverage, fuzz
regression, protocol fixtures, README examples, and dependency policy. See
[`DEVELOPMENT.md`](DEVELOPMENT.md).

---

## Security

Report vulnerabilities privately according to [`SECURITY.md`](SECURITY.md).
The server forbids `unsafe` code, parses untrusted buffers through bounded core
APIs, does not execute YAML tags, and communicates only through the configured
stdio child-process channel.

---

## Documentation

- [User Manual](https://sebastienrousseau.github.io/noyalib-lsp/manual/)
- [API reference](https://docs.rs/noyalib-lsp)
- [Developer documentation](DEVELOPMENT.md)
- [Ecosystem map](https://github.com/sebastienrousseau/noyalib/blob/main/docs/ECOSYSTEM.md)
- [Editor setup](docs/editor-setup.md)
- [Protocol coverage](docs/protocol-coverage.md)
- [Engineering policies](docs/POLICIES.md)
- [Compliance grade](docs/COMPLIANCE-GRADE.md)
- [Detailed README reference](docs/README-REFERENCE.md)

---

## Stability guarantees

- During `0.0.x`, the patch component is the breaking-change axis.
- Changes to diagnostics, edits, protocol capabilities, or configuration shape
  are treated as behavioural API changes.
- The LSP surface and core dependency release at the identical `0.0.x` version.
- The MSRV may rise only on the breaking axis with a changelog explanation.

---

## License

Licensed under either [Apache License 2.0](LICENSE-APACHE) or
[MIT](LICENSE-MIT), at your option.
