// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! End-to-end coverage for `full_document_edits` in
//! `noyalib-lsp::format`.
//!
//! `full_document_edits` normalizes via `noyalib::cst::format`, so
//! non-canonical input produces a real full-document `TextEdit` whose
//! `newText` is the *reformatted* document. (Before the v0.0.17 fix it
//! used a byte-faithful round-trip and always returned an empty edit —
//! the silent-no-op bug this file now guards against.)

use noyalib_lsp::format::full_document_edits;

#[test]
fn canonical_inputs_return_empty() {
    // Already-canonical documents are a no-op for the formatter, so the
    // server returns an empty edit list and the editor skips the edit.
    for input in [
        "a: 1\nb: 2\n",
        "key:\n  - one\n  - two\n",
        "name: foo\nport: 8080\n",
    ] {
        let edits = full_document_edits(input).expect("parse + format");
        assert!(
            edits.is_empty(),
            "canonical input must yield no edit: {input:?}"
        );
    }
}

#[test]
fn non_canonical_inputs_reformat() {
    // Each messy input must produce exactly one edit whose newText is
    // the normalized form — and never equal to the original bytes.
    let cases = [
        ("a:    1\nb:  2\n", "a: 1\nb: 2\n"),
        (
            "key1:    value\nkey2:    value\n",
            "key1: value\nkey2: value\n",
        ),
    ];
    for (messy, want) in cases {
        let edits = full_document_edits(messy).expect("parse + format");
        assert_eq!(edits.len(), 1, "expected one edit for {messy:?}");
        assert_eq!(
            edits[0]["newText"].as_str(),
            Some(want),
            "newText must be the normalized document for {messy:?}"
        );
        assert_ne!(
            edits[0]["newText"].as_str(),
            Some(messy),
            "edit must change the document (regression guard for the no-op bug)"
        );
    }
}

#[test]
fn end_line_calculation_for_multi_line() {
    let input = "key1:    value\nkey2:    value\nkey3:    value\n";
    let edits = full_document_edits(input).expect("ok");
    let end_line = edits[0]["range"]["end"]["line"].as_u64().unwrap();
    assert!(end_line >= 1, "multi-line doc must have end_line >= 1");
}

#[test]
fn single_line_no_trailing_newline() {
    // Exercises the `ends_with('\n') == false` and `lines().last()`
    // arms; the edit's end character is the length of the sole line.
    let edits = full_document_edits("foo:    bar").expect("ok");
    assert_eq!(edits.len(), 1);
    // Single line, no trailing newline → the `.max(1)` sentinel makes
    // end line 1 (the client clamps to the real end).
    assert_eq!(edits[0]["range"]["end"]["line"].as_u64(), Some(1));
    assert_eq!(edits[0]["newText"].as_str(), Some("foo: bar\n"));
}

#[test]
fn empty_input_is_ok_and_empty() {
    // Empty document parses and formats to itself → no edit.
    let edits = full_document_edits("").expect("ok");
    assert!(edits.is_empty());
}

#[test]
fn invalid_yaml_returns_error() {
    assert!(full_document_edits("a: [unclosed\n").is_err());
}
