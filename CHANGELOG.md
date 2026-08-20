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

## [v0.0.25] - 2026-08-20

Lockstep release with `noyalib` 0.0.25 — four fixes from @zoosky (#283,
#285, #288, #290), plus `remove` refusing an alias-valued entry instead
of silently doing nothing.

### Changed

- `noyalib` dependency pin `=0.0.24` -> `=0.0.25`, with the matching
  `cargo-vet` exemption moved alongside it.
- Crate version -> 0.0.25.
- Lockfile refreshed against the published core; only `noyalib` moved.

## [v0.0.25] - 2026-08-20

Lockstep release with `noyalib` 0.0.25. No behaviour change in this
crate — see the core's `CHANGELOG.md` for the four CST editor fixes
contributed by @zoosky and the differential-fuzz invariant correction.

### Changed

- `noyalib` dependency pin `=0.0.24` -> `=0.0.25`, with the matching
  `cargo-vet` exemption moved alongside it.
- Crate version -> 0.0.25.

## [v0.0.24] - 2026-08-19

Lockstep release with `noyalib` 0.0.24. No behaviour change in this
crate — see the core's `CHANGELOG.md`: `remove` now takes a sole entry's
head comment with it (#280), plus a dependency consolidation.

### Changed

- `noyalib` dependency pin `=0.0.23` -> `=0.0.24`, with the matching
  `cargo-vet` exemption moved alongside it.
- Crate version -> 0.0.24.
- Lockfile refreshed against the published core; only `noyalib` moved.

### Fixed

- Release assets now include the detached `.asc` signatures. The signing
  step produced them and `upload-artifact` carried them, but the
  `gh release create` call named every asset explicitly and omitted
  them, so they never reached the release. noyalib v0.0.24 shipped
  without signatures for this reason; the list is now a `nullglob`
  array, so the entries disappear when signing is skipped rather than
  failing the release.

## [v0.0.23] - 2026-08-16

Lockstep release with `noyalib` 0.0.23. No behaviour change in this
crate — see the core's `CHANGELOG.md` for what 0.0.23 carries: `remove`
extended to flow members and sole entries (closing #221), and
`swap_items` / `move_item` exchanging whole entries so comments travel
with the item they document (#269).

### Changed

- `noyalib` dependency pin `=0.0.22` -> `=0.0.23`, with the matching
  `cargo-vet` exemption moved alongside it.
- Crate version -> 0.0.23.
- Lockfile refreshed against the published core. Only `noyalib` moved —
  no new transitive dependencies, and no broad `cargo update`.

## [v0.0.22] - 2026-08-13

Lockstep release with `noyalib` 0.0.22. No behaviour change in this
crate — see the core's `CHANGELOG.md` for what 0.0.22 carries (CRLF-aware
CST splices, #261).

**On the version jump.** The published sequence for this crate goes
`0.0.18 → 0.0.22`. `0.0.19` was prepared on a release branch but never
tagged or published; `0.0.20` and `0.0.21` were core-only releases that
the satellites did not follow. Lockstep resumes here.

### Changed

- `noyalib` dependency pin `=0.0.18` → `=0.0.22`, with the matching
  `cargo-vet` exemption moved alongside it.
- Crate version → 0.0.22.

### Security

- Dropped the stale `RUSTSEC-2026-0173` ignore from `deny.toml`.
  `cargo-deny` reported it as `advisory-not-detected`: `proc-macro-error2`
  is not in this crate's graph on any platform, because it reaches
  `noyalib` only through the optional `validator` feature, which this
  crate does not enable. A stale ignore is not inert — it would have
  silently swallowed the advisory if a `validator`-enabled path were added
  later.

> **Undocumented gap.** `v0.0.17` and `v0.0.18` were released without
> entries here. Their contents are covered by the core's release notes for
> those versions.

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

- **MSRV 1.85.0 → 1.86.0**, matching the `noyalib` core floor. This is the
  lowest toolchain the crate can be **built and tested** on: `criterion
  0.8` (the benchmark dev-dependency) declares `rust-version = 1.86`, so
  `cargo check --all-targets` and the bench suite fail on 1.85
  (`criterion@0.8.2 requires rustc 1.86`), though `cargo check --lib`
  still builds. We publish the number we verify. If you consume only the
  library on 1.85, v0.0.15 remains available.
- `noyalib` dependency pin `=0.0.15` → `=0.0.16`.

### Internal

- Test coverage for `src/format.rs` rose from 46.48% to 100% of regions
  (47.22% → 100% of lines) as a direct consequence of the fix: the
  edit-construction path was previously unreachable, not merely untested.
  Crate totals are now 95.58% regions / 97.40% functions / 95.54% lines.
