// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `textDocument/formatting` — re-emit a YAML document via
//! noyalib's CST formatter and surface the result as LSP `TextEdit`
//! objects.
//!
//! The simplest correct implementation is "replace the entire
//! document range with the formatted output". That keeps the
//! response self-contained — the client doesn't need any
//! cross-document reasoning to apply the result.

use serde_json::{Value as JsonValue, json};

/// Build the LSP `TextEdit[]` array that, applied to `text`, yields
/// the formatted document.
///
/// Returns an empty array when `text` is already canonically
/// formatted; this lets the editor skip the no-op edit entirely.
///
/// # Errors
///
/// - The input fails to parse as YAML (the formatter has nothing
///   to emit until the document is syntactically valid).
pub fn full_document_edits(text: &str) -> noyalib::Result<Vec<JsonValue>> {
    // Must be `cst::format`, not `parse_document(..).to_string()`: the
    // CST round-trip is byte-faithful by design, so the latter always
    // compares equal to the input and this function would return an
    // empty edit list for every document — i.e. `textDocument/formatting`
    // would silently do nothing. `cst::format` is the call that actually
    // normalises whitespace while preserving comments.
    let formatted = noyalib::cst::format(text)?;
    if formatted == text {
        return Ok(Vec::new());
    }

    // LSP positions are zero-based line/character; the end is
    // *exclusive*. We use a sentinel large end so the range covers
    // the entire document regardless of length — the LSP spec
    // permits the server to clamp to the actual document end.
    let end_line = text
        .bytes()
        .filter(|&b| b == b'\n')
        .count()
        .max(1)
        .saturating_sub(if text.ends_with('\n') { 1 } else { 0 });
    let end_character = text.lines().last().unwrap_or("").len();

    Ok(vec![json!({
        "range": {
            "start": {"line": 0, "character": 0},
            "end":   {"line": end_line, "character": end_character},
        },
        "newText": formatted,
    })])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_canonical_input_returns_empty_edits() {
        let edits = full_document_edits("a: 1\nb: 2\n").unwrap();
        // The CST formatter is byte-faithful for already-canonical
        // input, so the response is the empty array.
        assert!(edits.is_empty());
    }

    #[test]
    fn unparseable_input_propagates_error() {
        let res = full_document_edits("a: [\n");
        assert!(res.is_err());
    }

    #[test]
    fn identity_input_produces_no_edits() {
        let edits = full_document_edits("simple: yaml\n").unwrap();
        assert!(edits.is_empty());
    }

    /// The edit-construction path. Previously unreachable: the
    /// implementation round-tripped the CST (byte-faithful), so
    /// `formatted == text` always held and this branch never ran.
    #[test]
    fn non_canonical_input_produces_one_full_range_edit() {
        let edits = full_document_edits("a:    1\nb:    2\n").unwrap();
        assert_eq!(edits.len(), 1, "expected a single whole-document edit");
        let e = &edits[0];
        assert_eq!(e["range"]["start"]["line"], 0);
        assert_eq!(e["range"]["start"]["character"], 0);
        assert!(e["range"]["end"]["line"].is_u64());
        assert!(e["range"]["end"]["character"].is_u64());
        assert_eq!(e["newText"], "a: 1\nb: 2\n");
    }

    #[test]
    fn end_position_for_trailing_newline_input() {
        let edits = full_document_edits("a:    1\nb:    2\n").unwrap();
        // Two lines, trailing newline: end line is the last content
        // line (zero-based), not the phantom line after it.
        assert_eq!(edits[0]["range"]["end"]["line"], 1);
    }

    /// Without a trailing newline the end line is a *sentinel* one past
    /// the last content line (`count(0).max(1)` with no newline to
    /// subtract). That is deliberate — the header comment notes the
    /// range intentionally over-reaches and the LSP spec lets the client
    /// clamp to the real document end. Asserted here so the behaviour is
    /// pinned rather than accidental.
    #[test]
    fn end_position_for_input_without_trailing_newline() {
        let edits = full_document_edits("a:    1").unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0]["range"]["end"]["line"], 1);
        // End character is the length of the last line of the *input*.
        assert_eq!(edits[0]["range"]["end"]["character"], "a:    1".len());
    }

    #[test]
    fn multi_line_nested_input_produces_edit() {
        let edits = full_document_edits("a:\n  - 1\n  -   2\n").unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0]["newText"], "a:\n  - 1\n  - 2\n");
    }
}
