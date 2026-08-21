// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Open a document, then mutate it — the sequence a live editor drives.
//!
//! `fuzz_handle_message` mostly exercises JSON framing. This one gets past
//! framing and varies the *YAML*, which is where the parser, the diagnostics
//! pass and the span mapping actually run. It also drives two messages
//! against one `Server`, so state carried between them is in scope rather
//! than reset each iteration.

#![no_main]

use libfuzzer_sys::fuzz_target;
use noyalib_lsp::Server;

fn json_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            c if (c as u32) < 0x20 => format!("\\u{:04x}", c as u32).chars().collect(),
            c => vec![c],
        })
        .collect()
}

fuzz_target!(|data: &[u8]| {
    let Ok(s) = core::str::from_utf8(data) else {
        return;
    };
    let text = json_escape(s);
    let mut server = Server::new();

    let open = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///f.yaml","languageId":"yaml","version":1,"text":"{text}"}}}}}}"#
    );
    let _ = server.handle_message(&open);

    let change = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"file:///f.yaml","version":2}},"contentChanges":[{{"text":"{text}"}}]}}}}"#
    );
    let _ = server.handle_message(&change);
});
