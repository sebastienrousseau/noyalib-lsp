<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Benchmarks

The `lsp_handlers` Criterion harness measures representative handler paths.

```bash
cargo bench --bench lsp_handlers
```

Record the CPU, operating system, Rust version, commit, document size, feature
set, and complete command with results. Compare runs on the same machine and
power profile. CI builds benchmark targets as a regression smoke test; shared
runner timing is not a release gate.
