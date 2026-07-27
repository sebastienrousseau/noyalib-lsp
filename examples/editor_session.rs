// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! A full editor session, driven in-process.
//!
//! The binary speaks `Content-Length`-framed JSON-RPC over stdio, so
//! the only way to watch it work is normally to attach a real editor
//! and read its LSP trace. This example drives the same [`Server`] the
//! stdio loop wraps, so every request and every notification is
//! visible.
//!
//! The sequence is what a client actually performs: `initialize`,
//! `didOpen` (which pushes diagnostics unprompted), an edit via
//! `didChange` that fixes the error and re-publishes an empty
//! diagnostic set, `hover`, `formatting`, then `didClose`, `shutdown`
//! and `exit` — plus the error responses a client must tolerate.
//!
//! Run: `cargo run --example editor_session`

use noyalib_lsp::{HandleOutcome, Server};

const URI: &str = "file:///tmp/config.yaml";

/// Send one message and show what came back.
fn send(server: &mut Server, label: &str, raw: &str) {
    println!("\n── {label} ──");
    let outcome: HandleOutcome = server.handle_message(raw);
    // A server may answer a request, push an unsolicited notification,
    // or stay silent. A client has to handle all three.
    if let Some(reply) = outcome.reply.as_deref() {
        println!("  <- reply:  {}", truncate(reply));
    }
    for note in &outcome.notifications {
        println!("  <- notify: {}", truncate(note));
    }
    if outcome.reply.is_none() && outcome.notifications.is_empty() {
        println!("  <- (silent)");
    }
}

fn truncate(s: &str) -> String {
    if s.len() > 240 {
        format!("{}… ({} bytes)", &s[..240], s.len())
    } else {
        s.to_string()
    }
}

fn main() {
    println!("noyalib-lsp — a full editor session, message by message");
    let mut server = Server::new();

    // 1. Handshake: the client learns which capabilities to enable.
    send(
        &mut server,
        "initialize",
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );

    // 2. The user opens a file containing a syntax error. `didOpen` is a
    //    notification (no id), yet the server answers with a
    //    `publishDiagnostics` push — that asymmetry is the part client
    //    authors most often get wrong.
    send(
        &mut server,
        "textDocument/didOpen — file with a syntax error",
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"{URI}","languageId":"yaml","version":1,"text":"server:\n  host: 0.0.0.0\n  ports: [80, 443\n"}}}}}}"#
        ),
    );
    println!("  open documents: {}", server.open_document_count());

    // 3. The user fixes the missing bracket. The server must re-publish
    //    an *empty* diagnostic array — not simply stop sending — or the
    //    stale squiggle never clears in the editor.
    send(
        &mut server,
        "textDocument/didChange — the error is fixed",
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"{URI}","version":2}},"contentChanges":[{{"text":"server:\n  host: 0.0.0.0\n  ports: [80, 443]\n"}}]}}}}"#
        ),
    );

    // 4. Hover over `host` to get the value card.
    send(
        &mut server,
        "textDocument/hover — over `host` (line 1)",
        &format!(
            r#"{{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{{"textDocument":{{"uri":"{URI}"}},"position":{{"line":1,"character":3}}}}}}"#
        ),
    );

    // 5. Formatting. Note this returns a real edit only because the
    //    document is non-canonical; an already-formatted file yields an
    //    empty array so the editor can skip the no-op.
    send(
        &mut server,
        "textDocument/didChange — introduce sloppy spacing",
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"{URI}","version":3}},"contentChanges":[{{"text":"a:    1\nb:    2\n"}}]}}}}"#
        ),
    );
    send(
        &mut server,
        "textDocument/formatting — returns one whole-document edit",
        &format!(
            r#"{{"jsonrpc":"2.0","id":3,"method":"textDocument/formatting","params":{{"textDocument":{{"uri":"{URI}"}},"options":{{"tabSize":2,"insertSpaces":true}}}}}}"#
        ),
    );

    println!("\n── error responses a client must tolerate ──");

    send(
        &mut server,
        "hover on a URI the server never opened",
        r#"{"jsonrpc":"2.0","id":4,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///nope.yaml"},"position":{"line":0,"character":0}}}"#,
    );
    send(
        &mut server,
        "unknown method",
        r#"{"jsonrpc":"2.0","id":5,"method":"textDocument/rename"}"#,
    );
    send(&mut server, "malformed JSON frame", r#"{not json"#);
    send(
        &mut server,
        "non-2.0 jsonrpc version",
        r#"{"jsonrpc":"1.0","id":6,"method":"initialize"}"#,
    );

    println!("\n── teardown ──");
    send(
        &mut server,
        "textDocument/didClose",
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didClose","params":{{"textDocument":{{"uri":"{URI}"}}}}}}"#
        ),
    );
    println!("  open documents: {}", server.open_document_count());

    send(
        &mut server,
        "shutdown",
        r#"{"jsonrpc":"2.0","id":7,"method":"shutdown"}"#,
    );
    // After `shutdown` the only legal message is `exit`; anything else
    // must be rejected rather than served.
    send(
        &mut server,
        "a request after shutdown — must be rejected",
        r#"{"jsonrpc":"2.0","id":8,"method":"textDocument/hover"}"#,
    );
    send(&mut server, "exit", r#"{"jsonrpc":"2.0","method":"exit"}"#);

    println!("\nSession complete.");
}
