<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# noyalib-lsp v0.0.16 Release Notes

Lockstep release with `noyalib` v0.0.16 (ADR-0005 strict-lockstep: the
LSP server publishes `=X.Y.Z` pinned to the core).

## What changed

- **`noyalib` pin `=0.0.15` → `=0.0.16`** and the crate version bumped
  in lockstep.
- **MSRV raised 1.85.0 → 1.86.0**, matching the single lockstep floor
  adopted in v0.0.16. The crate builds cleanly on 1.86.
- **Release workflow** publish steps made idempotent for clean re-runs.

## Known issues at v0.0.16 — corrected after release (ships in v0.0.17)

Two items were **not** in the v0.0.16 tag and are on `main` for the next
release. They are called out here so the record is accurate:

- **`textDocument/formatting` was a silent no-op.** `full_document_edits`
  used a byte-faithful CST round-trip (`parse_document().to_string()`),
  which equals its input for every parseable document, so the server
  always returned an empty edit list — format-on-save did nothing. The
  parent changelog stated this was fixed in v0.0.16; it was not. It is
  now fixed on `main` by calling `noyalib::cst::format`, with regression
  tests asserting a non-canonical document produces a real normalizing
  edit.
- **`crossbeam-epoch` RUSTSEC-2026-0204** (invalid-pointer-dereference)
  was present in the v0.0.16 lockfile via a transitive dependency and
  bumped to the patched 0.9.20 on `main` afterward.

## Engineering / CI (post-release, no user-facing change)

- Signed-history enforcement, upstream audit imports, and a
  `dependabot-vet` auto-refresh workflow.
- New CI gates brought to parity with the core: a coverage gate
  (measured 95.5 % regions / 97.3 % functions / 95.5 % lines against the
  93/96/94 floor), an MSRV gate, CodeQL, and OpenSSF Scorecard.

## What did not change

- The JSON-RPC / stdio LSP transport and message framing.
- `#![forbid(unsafe_code)]` — intact.

## Upgrading

```toml
noyalib-lsp = "0.0.16"
```

Requires **Rust 1.86.0+**. If you rely on `textDocument/formatting`,
wait for **v0.0.17**, where it actually reformats.
