// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `handle_message` is the language server's trust boundary.
//!
//! Every byte comes from an editor over stdio, and a panic there does not
//! surface as a diagnostic — it takes the server down mid-session and the
//! editor reports nothing useful. The invariant is that it returns for any
//! input; an error reply is a fine outcome, an abort is not.

#![no_main]

use libfuzzer_sys::fuzz_target;
use noyalib_lsp::Server;

fuzz_target!(|data: &[u8]| {
    let Ok(raw) = core::str::from_utf8(data) else {
        return;
    };
    let mut server = Server::new();
    let _ = server.handle_message(raw);
});
