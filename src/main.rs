// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `noyalib-lsp` — Language Server Protocol implementation for
//! noyalib. Stdio transport with the standard LSP framing
//! (`Content-Length` headers).
//!
//! This binary is the transport shim around [`noyalib_lsp::Server`].
//! All handler logic lives in the library so `cargo test` covers
//! it directly without standing up a real LSP client.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::io::{self, Read, Write};
use std::process::ExitCode;

use noyalib_lsp::Server;

const HELP: &str = "\
noyalib-lsp — Language Server Protocol implementation for noyalib.

USAGE:
  noyalib-lsp                   Start the LSP stdio server (the
                                normal mode an editor invokes).
  noyalib-lsp --version | -V    Print version and exit.
  noyalib-lsp --help | -h       Print this help and exit.

NOTES:
  This binary speaks the standard LSP wire format with
  `Content-Length` framing over stdio. It is not designed for
  interactive use — configure your editor to spawn it. Example for
  Neovim with `lspconfig`:

    require('lspconfig.configs').noyalib = {
      default_config = {
        cmd = { 'noyalib-lsp' },
        filetypes = { 'yaml' },
        root_dir = require('lspconfig.util').find_git_ancestor,
      },
    }
    require('lspconfig').noyalib.setup {}

REPORTING BUGS:
  https://github.com/sebastienrousseau/noyalib/issues
";

fn main() -> ExitCode {
    // Honour the conventional `--version` / `--help` flags before
    // falling into the LSP stdio loop. Without these, a user
    // running `noyalib-lsp` to verify the install just sees a hung
    // process; printing version / help is the standard CLI hygiene.
    if let Some(arg) = std::env::args().nth(1) {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("noyalib-lsp {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--help" | "-h" => {
                print!("{HELP}");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("noyalib-lsp: unknown argument `{other}`");
                eprintln!("Run `noyalib-lsp --help` for usage.");
                return ExitCode::from(2);
            }
        }
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("noyalib-lsp: {e}");
            ExitCode::from(3)
        }
    }
}

fn run() -> io::Result<()> {
    serve(io::stdin().lock(), io::stdout().lock())
}

/// Largest message body the server accepts. A header declaring more is
/// answered with a parse error and its body is skipped unread.
const MAX_MESSAGE: usize = 256 * 1024 * 1024;

/// Largest header block accepted before its terminating blank line.
const MAX_HEADER: usize = 8 * 1024;

/// Serve LSP messages from `input` until EOF, writing replies to `out`.
///
/// A malformed frame (no or an unparseable `Content-Length`, a declared
/// length over [`MAX_MESSAGE`], an oversized header, a body that is not
/// UTF-8) is answered with a JSON-RPC parse error (`-32700`, `id: null`)
/// and the server carries on with the next message.
fn serve(mut input: impl Read, mut out: impl Write) -> io::Result<()> {
    let mut server = Server::new();
    let mut framer = Framer::new(MAX_MESSAGE);
    let mut buf = [0u8; 4096];
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        framer.push(&buf[..n]);
        while let Some(frame) = framer.next_frame() {
            for message in respond(&mut server, frame) {
                write_message(&mut out, &message)?;
            }
            out.flush()?;
        }
    }
}

/// The messages to send back for one frame.
fn respond(server: &mut Server, frame: Frame) -> Vec<String> {
    let body = match frame {
        Frame::Refused(why) => return vec![parse_error(&why)],
        Frame::Body(bytes) => bytes,
    };
    let Ok(text) = std::str::from_utf8(&body) else {
        return vec![parse_error("message body is not valid UTF-8")];
    };
    let outcome = server.handle_message(text);
    outcome
        .reply
        .into_iter()
        .chain(outcome.notifications)
        .collect()
}

fn parse_error(why: &str) -> String {
    noyalib_lsp::error_str(
        serde_json::Value::Null,
        -32700,
        format!("parse error: {why}"),
    )
}

/// One unit cut from the byte stream.
#[derive(Debug, PartialEq)]
enum Frame {
    /// A complete message body.
    Body(Vec<u8>),
    /// A frame that could not be accepted, and why.
    Refused(String),
}

/// Splits the input stream into `Content-Length` framed messages.
struct Framer {
    pending: Vec<u8>,
    /// Length of the body whose header has been read, while its bytes
    /// are still arriving.
    body_len: Option<usize>,
    /// Bytes of a refused body still to drop as they arrive.
    skip: usize,
    max_message: usize,
}

impl Framer {
    fn new(max_message: usize) -> Self {
        Self {
            pending: Vec::new(),
            body_len: None,
            skip: 0,
            max_message,
        }
    }

    fn push(&mut self, bytes: &[u8]) {
        let dropped = self.skip.min(bytes.len());
        self.skip -= dropped;
        self.pending.extend_from_slice(&bytes[dropped..]);
    }

    /// The next complete frame, or `None` until more input arrives.
    fn next_frame(&mut self) -> Option<Frame> {
        if self.body_len.is_none() {
            match self.take_header()? {
                Ok(length) => self.body_len = Some(length),
                Err(refused) => return Some(refused),
            }
        }
        let length = self.body_len?;
        if self.pending.len() < length {
            return None;
        }
        self.body_len = None;
        Some(Frame::Body(self.pending.drain(..length).collect()))
    }

    /// Consume one header block: `Ok(length)` of the body that follows,
    /// `Err` with the refusal for a header that cannot be honoured, or
    /// `None` while the header is incomplete.
    fn take_header(&mut self) -> Option<Result<usize, Frame>> {
        let Some(header_end) = find_header_end(&self.pending) else {
            if self.pending.len() > MAX_HEADER {
                self.pending.clear();
                return Some(Err(Frame::Refused("header too large".into())));
            }
            return None;
        };
        let declared = content_length(&self.pending[..header_end]);
        self.pending.drain(..header_end);
        match declared {
            None => Some(Err(Frame::Refused(
                "missing or invalid Content-Length".into(),
            ))),
            Some(length) if length > self.max_message => {
                self.discard(length);
                let max = self.max_message;
                Some(Err(Frame::Refused(format!(
                    "Content-Length {length} exceeds the {max} byte limit"
                ))))
            }
            Some(length) => Some(Ok(length)),
        }
    }

    /// Drop the next `length` bytes, whether buffered or still to come.
    fn discard(&mut self, length: usize) {
        let now = length.min(self.pending.len());
        self.pending.drain(..now);
        self.skip = length - now;
    }
}

/// The declared `Content-Length` of a header block, if it has a valid one.
fn content_length(header: &[u8]) -> Option<usize> {
    let header = std::str::from_utf8(header).ok()?;
    header.lines().find_map(|line| {
        line.strip_prefix("Content-Length:")
            .and_then(|n| n.trim().parse::<usize>().ok())
    })
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 3 < bytes.len() {
        if &bytes[i..i + 4] == b"\r\n\r\n" {
            return Some(i + 4);
        }
        i += 1;
    }
    None
}

fn write_message(out: &mut impl Write, body: &str) -> io::Result<()> {
    write!(out, "Content-Length: {}\r\n\r\n{}", body.len(), body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_length_is_read_from_the_header() {
        assert_eq!(content_length(b"Content-Length: 42\r\n\r\n"), Some(42));
        let extra = b"Content-Type: application/json\r\nContent-Length: 7\r\n\r\n";
        assert_eq!(content_length(extra), Some(7));
        assert_eq!(content_length(b"Other-Header: x\r\n\r\n"), None);
        assert_eq!(content_length(b"Content-Length: -1\r\n\r\n"), None);
    }

    fn frames(framer: &mut Framer, input: &[u8]) -> Vec<Frame> {
        framer.push(input);
        std::iter::from_fn(|| framer.next_frame()).collect()
    }

    #[test]
    fn a_body_split_across_reads_is_reassembled() {
        let mut f = Framer::new(MAX_MESSAGE);
        assert!(frames(&mut f, b"Content-Length: 5\r\n\r\nab").is_empty());
        assert_eq!(frames(&mut f, b"cde"), vec![Frame::Body(b"abcde".to_vec())]);
    }

    #[test]
    fn an_oversized_body_is_refused_and_skipped_then_framing_resumes() {
        let mut f = Framer::new(4);
        let got = frames(&mut f, b"Content-Length: 10\r\n\r\n0123");
        assert!(matches!(&got[..], [Frame::Refused(_)]), "{got:?}");
        // The remaining six body bytes arrive and are dropped unread.
        let got = frames(&mut f, b"456789Content-Length: 2\r\n\r\nok");
        assert_eq!(got, vec![Frame::Body(b"ok".to_vec())]);
    }

    #[test]
    fn content_length_of_usize_max_is_refused_not_added() {
        let mut f = Framer::new(MAX_MESSAGE);
        let input = format!("Content-Length: {}\r\n\r\n{{}}", usize::MAX);
        let got = frames(&mut f, input.as_bytes());
        assert!(matches!(&got[..], [Frame::Refused(_)]), "{got:?}");
    }

    #[test]
    fn a_header_that_never_ends_is_refused_once_it_is_too_large() {
        let mut f = Framer::new(MAX_MESSAGE);
        let got = frames(&mut f, &vec![b'x'; MAX_HEADER + 1]);
        assert!(matches!(&got[..], [Frame::Refused(_)]), "{got:?}");
        assert!(f.pending.is_empty());
    }

    #[test]
    fn serve_answers_bad_frames_and_keeps_going() {
        let init = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let mut input = b"Content-Length: 3\r\n\r\n\xff\xfe\xfd".to_vec();
        input.extend(format!("Content-Length: {}\r\n\r\n", init.len()).as_bytes());
        input.extend(init);
        let mut out = Vec::new();
        serve(&input[..], &mut out).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(out.contains("-32700"), "{out}");
        assert!(out.contains("capabilities"), "{out}");
    }

    #[test]
    fn find_header_end_locates_terminator() {
        assert_eq!(find_header_end(b"a\r\n\r\nb"), Some(5));
        assert_eq!(find_header_end(b"abc"), None);
    }

    #[test]
    fn write_message_uses_content_length_prefix() {
        let mut out = Vec::new();
        write_message(&mut out, r#"{"hello":"world"}"#).unwrap();
        let s = std::str::from_utf8(&out).unwrap();
        assert!(s.starts_with("Content-Length: 17\r\n\r\n"));
        assert!(s.ends_with(r#"{"hello":"world"}"#));
    }
}
