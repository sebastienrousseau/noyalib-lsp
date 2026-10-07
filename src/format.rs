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

/// The LSP position just past the last character of `text`.
fn end_position(text: &str) -> (usize, usize) {
    let lines = text.bytes().filter(|&b| b == b'\n').count();
    let last_line = text.rsplit('\n').next().unwrap_or("");
    (lines, last_line.encode_utf16().count())
}

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

    // LSP positions are zero-based lines and UTF-16 code units, and the
    // end is exclusive. The range ends exactly at the end of the text:
    // after the final newline when there is one (so the edit replaces
    // it rather than leaving a duplicate behind), measured in UTF-16
    // so a line with non-ASCII characters is not over- or under-shot.
    let (end_line, end_character) = end_position(text);

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
        // Two lines and a trailing newline: the range ends at the start
        // of the empty line after it, so the newline is replaced too.
        assert_eq!(edits[0]["range"]["end"]["line"], 2);
        assert_eq!(edits[0]["range"]["end"]["character"], 0);
    }

    #[test]
    fn end_position_for_input_without_trailing_newline() {
        let edits = full_document_edits("a:    1").unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0]["range"]["end"]["line"], 0);
        assert_eq!(edits[0]["range"]["end"]["character"], "a:    1".len());
    }

    #[test]
    fn end_character_counts_utf16_code_units() {
        // "é" is 2 UTF-8 bytes and 1 UTF-16 unit; "😀" is 4 bytes and 2 units.
        assert_eq!(end_position("k: é😀"), (0, 6));
        assert_eq!(end_position("a\nk: é😀"), (1, 6));
        assert_eq!(end_position(""), (0, 0));
    }

    /// Apply a whole-document edit the way a client does: positions are
    /// zero-based lines and UTF-16 code units, and the range must cover
    /// exactly the text it replaces.
    fn apply(text: &str, edit: &JsonValue) -> String {
        let at = |pos: &JsonValue| -> usize {
            let line = pos["line"].as_u64().unwrap() as usize;
            let character = pos["character"].as_u64().unwrap() as usize;
            let line_start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
            let rest = &text[line_start..];
            let mut units = 0;
            for (i, c) in rest.char_indices() {
                if units == character {
                    return line_start + i;
                }
                assert!(
                    c != '\n',
                    "character {character} is past the end of line {line}"
                );
                units += c.len_utf16();
            }
            assert_eq!(units, character, "position past the end of the document");
            text.len()
        };
        let start = at(&edit["range"]["start"]);
        let end = at(&edit["range"]["end"]);
        format!(
            "{}{}{}",
            &text[..start],
            edit["newText"].as_str().unwrap(),
            &text[end..]
        )
    }

    #[test]
    fn applying_the_edit_yields_exactly_the_formatted_text() {
        for text in [
            "a:    1\nb:    2\n",
            "a:    1",
            "k:    \"é€😀\"\n",
            "k:    \"é€😀\"",
            "a:\n  - 1\n  -   2\n\n",
        ] {
            let edits = full_document_edits(text).unwrap();
            assert_eq!(edits.len(), 1, "{text:?}");
            let want = noyalib::cst::format(text).unwrap();
            assert_eq!(apply(text, &edits[0]), want, "{text:?}");
        }
    }

    #[test]
    fn multi_line_nested_input_produces_edit() {
        let edits = full_document_edits("a:\n  - 1\n  -   2\n").unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0]["newText"], "a:\n  - 1\n  - 2\n");
    }
}
