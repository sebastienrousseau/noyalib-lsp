<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# noyalib-lsp v0.0.17 Release Notes

Lockstep release with `noyalib` v0.0.17 (ADR-0005: publishes `=0.0.17`
pinned to the core).

## What changed

- **`textDocument/formatting` is no longer a silent no-op.** It now
  calls `noyalib::cst::format` instead of a byte-faithful CST round-trip,
  so format-on-save actually reformats. Regression-tested.
- `noyalib` pin `=0.0.16` -> `=0.0.17`; crate version bumped in lockstep.

## Repository hardening (CI/docs only)

Coverage, MSRV, CodeQL, and OpenSSF Scorecard gates plus upstream cargo-vet
audit imports were added across the satellites in this cycle; branch
protection now makes commit signing unskippable. No user-facing change.

## Upgrading

Requires **Rust 1.86.0+**.
