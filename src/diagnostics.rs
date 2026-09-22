// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `textDocument/publishDiagnostics` — turn YAML parse errors into
//! LSP-compatible diagnostic objects.
//!
//! The LSP wire shape is documented at
//! <https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#publishDiagnosticsParams>.

use noyalib::{DiagnosticSeverity, SourceSpan};
use serde_json::{Value as JsonValue, json};

/// LSP severity levels per the spec.
const SEVERITY_ERROR: i32 = 1;
const SEVERITY_WARNING: i32 = 2;
const SEVERITY_INFORMATION: i32 = 3;
const SEVERITY_HINT: i32 = 4;

/// Compose the JSON-RPC `textDocument/publishDiagnostics`
/// notification for `uri` with the diagnostics derived from `text`.
///
/// Returns `None` when there is nothing to publish (no parse error).
/// The caller forwards the returned string to stdout when present.
#[must_use]
pub fn publish_diagnostics(uri: &str, text: &str) -> Option<String> {
    let diagnostics = collect(text);
    // Always publish — the LSP spec requires the server to emit the
    // (possibly empty) list so the client can clear stale diagnostics.
    let params = json!({
        "uri": uri,
        "diagnostics": diagnostics,
    });
    let note = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": params,
    });
    Some(note.to_string())
}

/// Collect raw diagnostics from a YAML document. Public so tests
/// and richer integrations can inspect the diagnostic list before
/// the JSON-RPC envelope is built.
#[must_use]
pub fn collect(text: &str) -> Vec<JsonValue> {
    let mut diagnostics = Vec::new();
    // A buffer may hold a whole stream (`---`-separated documents);
    // `load_all_as` accepts every document count, so a valid
    // multi-document file never produces a false parse error.
    if let Err(err) = noyalib::load_all_as::<noyalib::Value>(text) {
        let diagnostic = err.diagnostic();
        let (start, end) = diagnostic
            .primary_label()
            .map_or_else(default_range, |label| lsp_range(text, label.span()));
        diagnostics.push(json!({
            "range": {
                "start": {"line": start.0, "character": start.1},
                "end":   {"line": end.0, "character": end.1},
            },
            "severity": severity_number(diagnostic.severity()),
            "code": diagnostic.code().as_str(),
            "source": "noyalib",
            "message": diagnostic.message(),
        }));
    }
    diagnostics
}

fn severity_number(severity: DiagnosticSeverity) -> i32 {
    match severity {
        DiagnosticSeverity::Error => SEVERITY_ERROR,
        DiagnosticSeverity::Warning => SEVERITY_WARNING,
        DiagnosticSeverity::Information => SEVERITY_INFORMATION,
        DiagnosticSeverity::Hint => SEVERITY_HINT,
        _ => SEVERITY_ERROR,
    }
}

fn default_range() -> ((usize, usize), (usize, usize)) {
    ((0, 0), (0, 1))
}

/// Convert a UTF-8 byte span into the zero-based UTF-16 coordinates required
/// by LSP 3.17. Offsets are clamped to scalar boundaries so stale spans cannot
/// panic or split a multi-byte character.
fn lsp_range(text: &str, span: SourceSpan) -> ((usize, usize), (usize, usize)) {
    let start_offset = floor_char_boundary(text, span.offset().min(text.len()));
    let mut end_offset = span.end().min(text.len());
    while end_offset < text.len() && !text.is_char_boundary(end_offset) {
        end_offset += 1;
    }
    if end_offset <= start_offset && start_offset < text.len() {
        end_offset = start_offset
            + text[start_offset..]
                .chars()
                .next()
                .map_or(1, char::len_utf8);
    }
    (
        lsp_position(text, start_offset),
        lsp_position(text, end_offset),
    )
}

fn floor_char_boundary(text: &str, mut offset: usize) -> usize {
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn lsp_position(text: &str, offset: usize) -> (usize, usize) {
    let prefix = &text[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let character = text[line_start..offset].encode_utf16().count();
    (line, character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_accepts_a_multi_document_stream() {
        // yaml-test-suite 35KP and friends: a `---`-separated stream
        // is valid YAML and must not raise a false parse error.
        assert!(collect("--- a\n--- b\n...\n").is_empty());
        assert!(collect("%YAML 1.2\n---\nk: v\n---\n- 1\n").is_empty());
    }

    #[test]
    fn collect_reports_an_error_in_any_document_of_a_stream() {
        let d = collect("k: v\n---\nk: [\n");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0]["severity"].as_i64(), Some(1));
    }

    #[test]
    fn collect_returns_empty_on_valid_yaml() {
        assert!(collect("a: 1\nb: 2\n").is_empty());
    }

    #[test]
    fn collect_returns_one_diagnostic_per_parse_error() {
        let d = collect("a: [\n");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0]["severity"].as_i64(), Some(1));
        assert_eq!(d[0]["source"].as_str(), Some("noyalib"));
        assert_eq!(d[0]["code"].as_str(), Some("noyalib::parse"));
        assert!(!d[0]["message"].as_str().unwrap().is_empty());
    }

    #[test]
    fn publish_diagnostics_wraps_in_jsonrpc_envelope() {
        let s = publish_diagnostics("file:///tmp/a.yaml", "a: 1\n").unwrap();
        let v: JsonValue = serde_json::from_str(&s).unwrap();
        assert_eq!(v["jsonrpc"].as_str(), Some("2.0"));
        assert_eq!(
            v["method"].as_str(),
            Some("textDocument/publishDiagnostics"),
        );
        assert_eq!(v["params"]["uri"].as_str(), Some("file:///tmp/a.yaml"));
        let diags = v["params"]["diagnostics"].as_array().unwrap();
        assert!(diags.is_empty());
    }

    #[test]
    fn publish_diagnostics_includes_errors_for_invalid_yaml() {
        let s = publish_diagnostics("file:///tmp/a.yaml", "k: [\n").unwrap();
        let v: JsonValue = serde_json::from_str(&s).unwrap();
        let diags = v["params"]["diagnostics"].as_array().unwrap();
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn diagnostic_range_is_well_formed() {
        let d = collect("a: [\n");
        let range = &d[0]["range"];
        assert!(range["start"]["line"].is_u64());
        assert!(range["end"]["line"].is_u64());
    }

    #[test]
    fn diagnostic_uses_the_parser_source_location() {
        let d = collect("valid: true\ninvalid: [\n");
        let range = &d[0]["range"];
        assert_ne!(range["start"]["line"].as_u64(), Some(0));
    }

    #[test]
    fn byte_spans_are_converted_to_utf16_positions() {
        let text = "a😀b\nvalue";
        let start = text.find('b').unwrap();
        let (from, to) = lsp_range(text, SourceSpan::new(start, 1));
        assert_eq!(from, (0, 3));
        assert_eq!(to, (0, 4));

        let second_line = text.find("value").unwrap();
        assert_eq!(lsp_range(text, SourceSpan::new(second_line, 1)).0, (1, 0));
    }

    #[test]
    fn byte_spans_never_split_utf8_scalars() {
        let text = "😀";
        let (from, to) = lsp_range(text, SourceSpan::new(1, 1));
        assert_eq!(from, (0, 0));
        assert_eq!(to, (0, 2));
    }
}
