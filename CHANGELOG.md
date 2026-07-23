<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Changelog

All notable changes to `noyalib-lsp` are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
and versions in lockstep with the
[`noyalib`](https://github.com/sebastienrousseau/noyalib) core crate —
see that repository's `CHANGELOG.md` for the release-wide notes.

## [Unreleased]

(Nothing yet — `[v0.0.16]` is the cut.)

## [v0.0.16] - 2026-07-22

Lockstep release with `noyalib` 0.0.16. Contains one **user-facing bug
fix**: document formatting never actually worked.

### Fixed

- **`textDocument/formatting` was a silent no-op.** `full_document_edits`
  derived the formatted text from `cst::parse_document(text).to_string()`.
  That round-trip is byte-faithful by design — it reproduces the source
  exactly — so `formatted == text` held for every parseable document, the
  server always returned an empty `TextEdit[]`, and *no editor ever saw a
  formatting change*. The implementation now calls `noyalib::cst::format`,
  which normalises whitespace while preserving comments.

  Anyone who concluded that "Format Document" was broken against a YAML
  file was correct; it was. No configuration or client-side workaround was
  ever needed, and none should be kept.

  The defect was pinned by the test suite rather than caught by it:
  `tests/format_coverage.rs` described the edit-building code as
  unreachable and asserted the empty result as a "round-trip-empty
  contract". That test now asserts the opposite — non-canonical input must
  produce exactly one whole-document edit — so the no-op cannot return
  unnoticed.

### Changed

- **MSRV 1.85.0 → 1.86.0**, matching the `noyalib` core floor. This is a
  **deliberate policy choice**, not a dependency requirement — this crate
  still compiles on 1.85. The floor is raised so the whole lockstep set
  states one number, with headroom against a future transitive bump. If
  you are pinned to 1.85, v0.0.15 remains available.
- `noyalib` dependency pin `=0.0.15` → `=0.0.16`.

### Internal

- Test coverage for `src/format.rs` rose from 46.48% to 100% of regions
  (47.22% → 100% of lines) as a direct consequence of the fix: the
  edit-construction path was previously unreachable, not merely untested.
  Crate totals are now 95.58% regions / 97.40% functions / 95.54% lines.
