# Security Policy

## Supported Versions

| Version | Supported |
|:--------|:---------:|
| 0.0.x   | Yes       |

`noyalib-lsp` follows the [ADR-0005 strict-lockstep versioning
contract](https://github.com/sebastienrousseau/noyalib/blob/main/doc/adr/0005-workspace-split.md).
Every release is coordinated with `noyalib` at the same
version.

## Reporting a Vulnerability

Email **sebastian.rousseau@gmail.com**. Do not open a public
issue for security reports.

Include: description, steps to reproduce, affected versions,
suggested fix (optional). Initial response within 48h; fix or
mitigation plan within 7 days.

## Threat Model — LSP-Specific

`noyalib-lsp` speaks JSON-RPC 2.0 over stdio (per the Language
Server Protocol specification). Threat model:

- **Untrusted YAML documents in editor buffers**: every open,
  hover, format, and diagnostic operation feeds YAML through
  the same parser hardening path as the library crate.
  `max_depth`, `max_document_length`, `max_alias_expansions`,
  `max_mapping_keys`, `max_sequence_length` all apply.
- **Subprocess model**: `noyalib-lsp` runs as a child of the
  editor. It never opens listening sockets, never accepts
  network connections. `#[forbid(unsafe_code)]` workspace-wide.
- **JSON Schema resolution**: when `--schema` is used, the
  schema is fetched via the standard resolver — no untrusted
  network access unless the schema URL is untrusted.

## Security Design

Inherits every security invariant from parent `noyalib`:
`#[forbid(unsafe_code)]`, no C deps, parser DoS guards.

## Supply Chain

- `cargo-deny` in CI (advisories + bans + licenses + sources).
- All GitHub Actions SHA-pinned.
- CI composed from `sebastienrousseau/noyalib`'s shared
  reusable workflows.
- `Cargo.lock` committed for deterministic builds.

## Build Provenance & Artefact Signing

Each release ships with:

1. SLSA Level 3 build provenance via
   `actions/attest-build-provenance`.
2. Keyless sigstore signatures (Fulcio + Rekor) on every
   published `.crate`.
3. SBOM attached to each GitHub Release.

## Commit Integrity

Every commit on `main` must be signed. CI rejects unsigned PR
commits via `shared-verify-signatures.yml`.
