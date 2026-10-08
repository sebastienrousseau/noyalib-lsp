// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Malformed `Content-Length` framing must cost the client one JSON-RPC
//! parse error (-32700), never the server process.

#![allow(missing_docs)]

use std::io::Write;
use std::process::{Command, Stdio};

fn frame(body: &[u8]) -> Vec<u8> {
    let mut out = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    out.extend_from_slice(body);
    out
}

const INIT: &[u8] = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;

/// Feed `input` to the server, close stdin, return (exit code, stdout).
fn drive(input: &[u8]) -> (Option<i32>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_noyalib-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn content_length_of_u64_max_is_refused_without_a_panic() {
    let input = format!("Content-Length: {}\r\n\r\n{{}}", u64::MAX);
    let (code, stdout) = drive(input.as_bytes());
    assert_eq!(
        code,
        Some(0),
        "server must exit cleanly at EOF; stdout: {stdout}"
    );
    assert!(
        stdout.contains("-32700"),
        "expected a parse error reply: {stdout}"
    );
}

#[test]
fn invalid_utf8_body_gets_a_parse_error_and_the_server_continues() {
    let mut input = frame(b"{\"jsonrpc\":\"2.0\",\"method\":\"x\",\"params\":\"\xff\"}");
    input.extend(frame(INIT));
    let (code, stdout) = drive(&input);
    assert_eq!(code, Some(0), "stdout: {stdout}");
    assert!(stdout.contains("-32700"), "{stdout}");
    assert!(
        stdout.contains("\"capabilities\""),
        "initialize after the bad body was not answered: {stdout}"
    );
}

#[test]
fn header_without_content_length_is_refused_and_the_server_continues() {
    let mut input = b"Content-Type: application/json\r\n\r\n".to_vec();
    input.extend(frame(INIT));
    let (code, stdout) = drive(&input);
    assert_eq!(code, Some(0), "stdout: {stdout}");
    assert!(stdout.contains("-32700"), "{stdout}");
    assert!(stdout.contains("\"capabilities\""), "{stdout}");
}
