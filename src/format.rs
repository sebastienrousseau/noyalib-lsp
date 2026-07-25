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
/// - The input fails to parse as YAML (the formatter has nothing to emit until
///   the document is syntactically valid).
pub fn full_document_edits(text: &str) -> noyalib::Result<Vec<JsonValue>> {
    // Use `cst::format` (the normalizing formatter), NOT
    // `parse_document(text).to_string()`: the latter is a byte-faithful
    // round-trip, so `formatted == text` for every parseable input and
    // the server always returns an empty edit list. That was the
    // silent-no-op bug — `textDocument/formatting` never changed
    // anything in the editor.
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
        // `cst::format` is a no-op on already-canonical input, so the
        // response is the empty array and the editor skips the edit.
        let edits = full_document_edits("a: 1\nb: 2\n").unwrap();
        assert!(edits.is_empty());
    }

    #[test]
    fn unparseable_input_propagates_error() {
        let res = full_document_edits("a: [\n");
        assert!(res.is_err());
    }

    #[test]
    fn non_canonical_input_produces_a_reformatting_edit() {
        // Regression for the silent-no-op bug: messy spacing must
        // yield a real edit whose `newText` is the *normalized*
        // document — not the original bytes. This is the path that was
        // dead when the code round-tripped byte-faithfully.
        let messy = "a:    1\nb:  2\n";
        let edits = full_document_edits(messy).unwrap();
        assert_eq!(edits.len(), 1, "expected exactly one full-document edit");
        let e = &edits[0];
        assert_eq!(e["newText"].as_str(), Some("a: 1\nb: 2\n"));
        assert_ne!(
            e["newText"].as_str(),
            Some(messy),
            "the edit must change the document, or it is the no-op bug again"
        );
        // The range starts at the document origin and its end is a
        // well-formed zero-based position.
        assert_eq!(e["range"]["start"]["line"].as_u64(), Some(0));
        assert_eq!(e["range"]["start"]["character"].as_u64(), Some(0));
        assert!(e["range"]["end"]["line"].is_u64());
        assert!(e["range"]["end"]["character"].is_u64());
    }

    #[test]
    fn end_position_tracks_the_last_line_without_trailing_newline() {
        // Exercises the `end_line` / `end_character` computation on an
        // input that does NOT end in '\n' (the `saturating_sub(0)` and
        // `lines().last()` arms).
        let messy = "x:   1"; // no trailing newline, non-canonical spacing
        let edits = full_document_edits(messy).unwrap();
        assert_eq!(edits.len(), 1);
        // The document is normalized (spacing collapsed, trailing
        // newline added).
        assert_eq!(edits[0]["newText"].as_str(), Some("x: 1\n"));
        // `end_line` uses a deliberate `.max(1)` sentinel, so even a
        // single-line document reports end line 1 with no trailing
        // newline; the LSP client clamps it to the real document end.
        assert_eq!(edits[0]["range"]["end"]["line"].as_u64(), Some(1));
    }
}
