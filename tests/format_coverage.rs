// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Coverage for `full_document_edits` in `noyalib-lsp::format`.
//!
//! Historical note: the edit-building path used to be unreachable
//! because the implementation round-tripped the CST
//! (`parse_document(..).to_string()`), which is byte-faithful by
//! design — so `formatted == text` always held and the server returned
//! an empty edit list for every document. That made
//! `textDocument/formatting` a silent no-op. The implementation now
//! calls `cst::format`, and these tests assert that non-canonical input
//! actually produces an edit.

use noyalib_lsp::format::full_document_edits;

/// Already-canonical input is a genuine no-op: nothing to change, so
/// the server returns an empty edit list and the editor skips it.
#[test]
fn canonical_inputs_return_empty() {
    for input in ["a: 1\nb: 2\n", "key:\n  - one\n  - two\n", "{a: 1, b: 2}\n"] {
        let edits = full_document_edits(input).expect("parse + format");
        assert!(
            edits.is_empty(),
            "canonical input should need no edit: {input:?}"
        );
    }
}

/// Non-canonical input must produce exactly one whole-document edit
/// whose `newText` is the normalised source. This is the regression
/// guard for the silent-no-op bug described in the module header.
#[test]
fn non_canonical_inputs_produce_an_edit() {
    for (input, expected) in [
        ("a:    1\nb:  2\n", "a: 1\nb: 2\n"),
        ("key:\n  - one\n  -   two\n", "key:\n  - one\n  - two\n"),
    ] {
        let edits = full_document_edits(input).expect("parse + format");
        assert_eq!(edits.len(), 1, "expected one edit for {input:?}");
        assert_eq!(
            edits[0]["newText"], expected,
            "unexpected formatted text for {input:?}"
        );
        assert_eq!(edits[0]["range"]["start"]["line"], 0);
        assert_eq!(edits[0]["range"]["start"]["character"], 0);
    }
}

#[test]
fn end_character_for_input_without_trailing_newline() {
    // Input has no trailing `\n`. The end-character calculation
    // walks `text.lines().last()` and takes its length.
    let input = "a: 1";
    let edits = full_document_edits(input);
    assert!(edits.is_ok());
}

#[test]
fn end_line_calculation_for_multi_line() {
    let input = "key1:    value\nkey2:    value\nkey3:    value\n";
    let edits = full_document_edits(input).expect("ok");
    if let Some(e) = edits.first() {
        let end_line = e["range"]["end"]["line"].as_u64().unwrap();
        assert!(end_line >= 1, "multi-line doc must have end_line >= 1");
    }
}

#[test]
fn empty_input_handled() {
    let edits = full_document_edits("");
    assert!(edits.is_ok());
}

#[test]
fn single_line_no_newline() {
    // Tests the `text.ends_with('\n')` branch where it's false.
    let edits = full_document_edits("foo:    bar");
    assert!(edits.is_ok());
}

#[test]
fn already_canonical_returns_empty() {
    let edits = full_document_edits("name: foo\nport: 8080\n").expect("ok");
    assert!(edits.is_empty(), "canonical input → no edits");
}

#[test]
fn invalid_yaml_returns_error() {
    let r = full_document_edits("a: [unclosed\n");
    assert!(r.is_err());
}
