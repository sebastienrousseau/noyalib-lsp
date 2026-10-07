// SPDX-FileCopyrightText: 2026 Noyalib
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Which program the extension runs as the language server. Kept free of
// the `vscode` module so it can be tested with plain `node --test`.
"use strict";

const path = require("path");

const DEFAULT_COMMAND = "noyalib-lsp";

// Turn the `noyalib.path` setting into the command to spawn.
//
// A bare command name (no path separator) is looked up on PATH. Anything
// with a separator must be an absolute path: a relative one would resolve
// against whatever directory the server happens to start in, which is how
// a binary planted in an opened folder gets run.
//
// Returns `{ command }`, or `{ error }` with a message for the user.
function resolveServerCommand(value, platform = process.platform) {
  const raw = typeof value === "string" ? value.trim() : "";
  if (raw === "") {
    return { command: DEFAULT_COMMAND };
  }
  const windows = platform === "win32";
  const hasSeparator = raw.includes("/") || (windows && raw.includes("\\"));
  // On Windows "C:tool.exe" is relative to the drive's current directory.
  const driveRelative = windows && /^[A-Za-z]:(?![\\/])/.test(raw);
  if (!hasSeparator && !driveRelative) {
    return { command: raw };
  }
  const absolute = windows ? path.win32.isAbsolute(raw) && !driveRelative : path.posix.isAbsolute(raw);
  if (!absolute) {
    return {
      error: `noyalib.path must be a command name on PATH or an absolute path, not the relative path "${raw}".`,
    };
  }
  return { command: raw };
}

module.exports = { resolveServerCommand, DEFAULT_COMMAND };
