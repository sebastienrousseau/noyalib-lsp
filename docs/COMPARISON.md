<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Language-server comparison

| Surface | noyalib-lsp | Generic YAML language server | Native noyalib |
| :--- | :---: | :---: | :---: |
| Lossless CST edits | Yes | Implementation-dependent | Yes |
| JSON Schema 2020-12 diagnostics | Yes | Implementation-dependent | Library API |
| LSP stdio server | Yes | Yes | No |
| Shared parser with noya-cli | Yes | No | Yes |

This table compares integration shape, not completeness or performance. Consult
the upstream documentation and test the schema dialects, editor extensions, and
formatting behaviour required by a deployment.

See [protocol coverage](protocol-coverage.md) for the implemented methods and
[editor setup](editor-setup.md) for host configuration.
