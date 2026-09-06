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

## [v0.0.37] - 2026-09-06

### Changed

- Lockstep release with noyalib 0.0.37: the line opened for the
  noyalib-wasm npm package gate repair (its 0.0.36 npm publish failed
  inside the gate under npm 12). No local code change unless listed
  below.

## [v0.0.36] - 2026-09-06

### Changed

- Lockstep release with noyalib 0.0.36: stream parse errors located in
  the stream (core #408) and jsonschema 0.53 (core #405). No local code
  change unless listed below.

## [v0.0.35] - 2026-09-06

### Changed

- Lockstep release with noyalib 0.0.35: the 10/10 programme (formal
  budget proofs, a wasip2 build, scorecard hardening, the cookbook) and
  the family gaps closed in this cycle. No local code change unless
  listed below.

### Added

- A VS Code extension (`editors/vscode`): starts `noyalib-lsp` for YAML
  documents, a `noyalib.path` setting, a restart command, and a CI job
  that packages the `.vsix` on every push. The README previously named a
  marketplace listing that did not exist; it now describes the extension
  that does.

## [v0.0.34] - 2026-09-05

### Changed

- Lockstep release with noyalib 0.0.34: property tests for the emitter
  and path grammar, structure-aware and alloc-only fuzzing, the
  `arbitrary` feature, and an unterminated-verbatim-tag parser fix
  (core #396). No local code change unless listed below.
- yaml-test-suite conformance gate: `tests/yaml_test_suite.rs` drives all
  406 official cases through this crate's own entry point and CI runs it
  via the family's shared `yaml-test-suite` workflow, so the surface
  cannot drift from the core (which passes 406/406).

### Fixed

- Diagnostics and hover parse the buffer as a stream (`load_all_as`),
  so a valid multi-document file (`---`-separated) no longer receives
  a false "more than one document is not supported" error. Found by
  running the yaml-test-suite through the server: 19 valid stream
  cases were flagged. Hover on a stream reports the document count
  and the first document's type.

## [v0.0.33] - 2026-09-05

### Changed

- Repository hygiene for the family standard: the community files
  (code of conduct, governance, support, agent invariants, citation),
  `docs/ARCHITECTURE.md`, a rendered manual deployed to Pages, and a
  seed corpus replayed by CI on every push for the fuzz targets.
- Lockstep release with noyalib 0.0.33: bracket-quoted path segments
  (core #389), located duplicate-key errors (core #393), and serializer
  fixes for tag-like keys, non-printable characters, and block scalars
  (core #381, #391, #392). No local code change.

## [v0.0.32] - 2026-09-03

### Changed

- Lockstep release with noyalib 0.0.32: block sequence spans report
  their full extent (core #375). No local behaviour change.

## [v0.0.31] - 2026-09-03

### Changed

- **Repository layout, Phase 1 of the family structure plan**:
  `doc/` renamed to `docs/`, `DEVELOPMENT.md` added as the developer
  entry point, `.editorconfig` / `.markdownlint.yaml` /
  `.codespellrc` land with a per-push `docs-lint` CI gate consuming
  the core repo's shared-docs-lint.yml.

## [v0.0.30] - 2026-09-02

### Changed

- Lockstep release with noyalib 0.0.30 (exact serde_yaml location
  parity: tagged/anchored node spans anchor at their properties;
  the `custom-explicit-tag` contract case now pins `1:8:7`). No
  satellite-local changes.

## [v0.0.29] - 2026-09-01

### Added

- **CycloneDX SBOM in the release pipeline** (mirrors the core
  repo). Releases now emit a machine-readable CycloneDX 1.5
  `SBOM.cdx.json` — attested (SLSA), sigstore-signed, optionally
  GPG-signed, and attached to the GitHub Release — alongside the
  human-readable `SBOM.txt`, which was never a machine-readable
  SBOM format.

### Fixed

- **A GPG-less release could not publish.** The release asset list
  relied on `nullglob` to drop the `.asc` entries when GPG signing
  is skipped, but `artifacts/SBOM.txt.asc` was a literal path —
  `nullglob` only removes unmatched *patterns* — so
  `gh release create` failed on the missing file for any fork
  without the signing key. The entries are spelled as real globs
  now (mirrors the core repo's fix).

## [v0.0.28] - 2026-08-23

Lockstep release with the `noyalib` core. No changes in this crate; the
version moves so the `=0.0.28` pin resolves.

The core ships two correctness fixes around implicit nulls — inserting
over one appended a duplicate key, and a `:` at end of input was not
read as a value indicator. See the core's `CHANGELOG.md` for detail.

## [v0.0.27] - 2026-08-21

Lockstep release with `noyalib` 0.0.27. No behaviour change in this
crate, but the core carries one worth reading: only a **plain** `<<`
scalar is a merge key now — a quoted `"<<"`, and an alias resolving to
the string `<<`, are ordinary keys. A document relying on either
spelling to merge will stop merging, silently. See the core's
`CHANGELOG.md` for that and for @mathstuf's alias-resolution fix.

### Changed

- `noyalib` dependency pin `=0.0.26` -> `=0.0.27`, with both the
  `noyalib` and the self `cargo-vet` exemptions moved alongside it.
- Crate version -> 0.0.27.
- Lockfile refreshed against the published core; only `noyalib` moved.

## [v0.0.26] - 2026-08-20

Lockstep release with `noyalib` 0.0.26. No behaviour change in this
crate — see the core's `CHANGELOG.md` for @zoosky's wrapped-flow fix
(#294 / #296): a flow member alone on its line now takes the line with
it, so removing from a collection wrapped one member per line no longer
leaves a whitespace-only line behind.

### Changed

- `noyalib` dependency pin `=0.0.25` -> `=0.0.26`, with the matching
  `cargo-vet` exemption moved alongside it.
- Crate version -> 0.0.26.
- Lockfile refreshed against the published core; only `noyalib` moved.

## [v0.0.25] - 2026-08-20

Lockstep release with `noyalib` 0.0.25. No behaviour change in this
crate — see the core's `CHANGELOG.md` for the four CST editor fixes
contributed by @zoosky (#283, #285, #288, #290), `remove` refusing an
alias-valued entry instead of silently doing nothing, and the
differential-fuzz invariant correction.

### Changed

- `noyalib` dependency pin `=0.0.24` -> `=0.0.25`, with the matching
  `cargo-vet` exemption moved alongside it.
- Crate version -> 0.0.25.
- Lockfile refreshed against the published core; only `noyalib` moved.

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
