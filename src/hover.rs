// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `textDocument/hover` — surface contextual information for the
//! cursor position.
//!
//! The first iteration's contract: when the document parses
//! cleanly, return a small markdown card noting the line / column
//! and the document's overall type. When the document fails to
//! parse, surface the parse error in the hover so the user can
//! act on it without leaving the cursor.
//!
//! Future work tracked separately: when a JSON Schema is attached,
//! resolve the schema description for the field at the cursor.

use serde_json::{Value as JsonValue, json};

/// Hover result for the cursor at zero-based `line` / `column` in
/// `text`. Returns `null` when there is nothing to surface — the
/// LSP spec lets the server return `null` to mean "nothing to say
/// here".
#[must_use]
pub fn hover_at(text: &str, line: usize, column: usize) -> JsonValue {
    if byte_offset_of(text, line, column).is_none() {
        return JsonValue::Null;
    }
    let body = match noyalib::load_all_as::<noyalib::Value>(text) {
        Ok(docs) if docs.len() > 1 => format!(
            "**Position**: line {}, column {}\n\n**Documents**: {}\n\n**First document type**: `{}`",
            line + 1,
            column + 1,
            docs.len(),
            type_name(&docs[0]),
        ),
        Ok(docs) => format!(
            "**Position**: line {}, column {}\n\n**Document type**: `{}`",
            line + 1,
            column + 1,
            docs.first().map_or("null", type_name),
        ),
        Err(e) => format!("**Parse error**\n\n```\n{e}\n```"),
    };
    json!({
        "contents": {
            "kind": "markdown",
            "value": body,
        }
    })
}

/// Convert a zero-based LSP `(line, column)` using UTF-16 code units into
/// a UTF-8 byte offset. Returns `None` outside the requested line or inside
/// a surrogate pair. CRLF, LF and CR each delimit one line.
pub fn byte_offset_of(text: &str, line: usize, column: usize) -> Option<usize> {
    let mut remaining = text;
    let mut line_start = 0;
    for _ in 0..line {
        let end = remaining.find(['\r', '\n'])?;
        let break_len = if remaining[end..].starts_with("\r\n") {
            2
        } else {
            1
        };
        line_start += end + break_len;
        remaining = &remaining[end + break_len..];
    }
    let end = remaining.find(['\r', '\n']).unwrap_or(remaining.len());
    let line_text = &remaining[..end];
    let mut units = 0;
    for (offset, character) in line_text.char_indices() {
        if units == column {
            return Some(line_start + offset);
        }
        units += character.len_utf16();
        if units > column {
            return None;
        }
    }
    (units == column).then_some(line_start + line_text.len())
}

fn type_name(v: &noyalib::Value) -> &'static str {
    match v {
        noyalib::Value::Null => "null",
        noyalib::Value::Bool(_) => "bool",
        noyalib::Value::Number(_) => "number",
        noyalib::Value::String(_) => "string",
        noyalib::Value::Sequence(_) => "sequence",
        noyalib::Value::Mapping(_) => "mapping",
        noyalib::Value::Tagged(_) => "tagged",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_cr_and_empty_final_lines_have_protocol_positions() {
        assert_eq!(byte_offset_of("a\rb", 1, 1), Some(3));
        assert_eq!(byte_offset_of("a\r\n", 1, 0), Some(3));
        assert_eq!(byte_offset_of("a\r\n", 2, 0), None);
        assert_eq!(byte_offset_of("", 0, 0), Some(0));
    }

    #[test]
    fn positions_count_utf16_units_and_stay_on_the_requested_line() {
        let text = "😀a\r\nb: 1\n";
        assert_eq!(byte_offset_of(text, 0, 2), Some(4));
        assert_eq!(byte_offset_of(text, 0, 3), Some(5));
        assert_eq!(byte_offset_of(text, 0, 4), None);
        assert_eq!(byte_offset_of(text, 0, 1), None);
        assert_eq!(byte_offset_of(text, 1, 0), Some(7));
    }

    #[test]
    fn ascii_columns_cannot_spill_into_the_next_line() {
        assert_eq!(byte_offset_of("a\nb: 1", 0, 2), None);
        assert_eq!(hover_at("a\nb: 1", 0, 2), JsonValue::Null);
    }

    #[test]
    fn hover_describes_a_multi_document_stream() {
        let v = hover_at("a: 1\n---\n- x\n", 0, 0);
        let body = v["contents"]["value"].as_str().unwrap();
        assert!(body.contains("**Documents**: 2"), "{body}");
        assert!(
            body.contains("**First document type**: `mapping`"),
            "{body}"
        );
    }

    #[test]
    fn hover_returns_null_for_out_of_range_position() {
        assert_eq!(hover_at("a: 1\n", 99, 99), JsonValue::Null);
    }

    #[test]
    fn hover_at_well_formed_input_returns_markdown_card() {
        let v = hover_at("name: noyalib\n", 0, 0);
        assert_eq!(v["contents"]["kind"].as_str(), Some("markdown"));
        let body = v["contents"]["value"].as_str().unwrap();
        assert!(body.contains("mapping"));
        assert!(body.contains("line 1"));
    }

    #[test]
    fn hover_surfaces_parse_error_when_input_invalid() {
        let v = hover_at("a: [\n", 0, 0);
        assert_eq!(v["contents"]["kind"].as_str(), Some("markdown"));
        let body = v["contents"]["value"].as_str().unwrap();
        assert!(body.contains("Parse error"));
    }

    #[test]
    fn byte_offset_of_handles_multi_line_input() {
        let text = "abc\ndef\nghi\n";
        assert_eq!(byte_offset_of(text, 1, 1), Some(5));
        assert_eq!(byte_offset_of(text, 0, 0), Some(0));
    }

    #[test]
    fn byte_offset_of_returns_none_for_out_of_range_line() {
        assert_eq!(byte_offset_of("a\n", 5, 0), None);
    }

    #[test]
    fn byte_offset_of_a_huge_column_is_out_of_range_not_an_overflow() {
        assert_eq!(byte_offset_of("a: 1\nb: 2\n", 1, usize::MAX), None);
        assert_eq!(byte_offset_of("abc", 0, usize::MAX), None);
        assert_eq!(hover_at("a: 1\n", 0, usize::MAX), JsonValue::Null);
    }

    #[test]
    fn byte_offset_of_handles_single_line_with_no_trailing_newline() {
        // "abc" — no newline. Line 0 is the entire string.
        assert_eq!(byte_offset_of("abc", 0, 2), Some(2));
        // Past the end of line 0 — out of range.
        assert_eq!(byte_offset_of("abc", 0, 99), None);
    }
}
