// SPDX-FileCopyrightText: 2026 Noyalib
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// node --test test/
"use strict";

const test = require("node:test");
const assert = require("node:assert");
const { resolveServerCommand } = require("../src/server-command");

test("a bare command name is looked up on PATH", () => {
  assert.deepStrictEqual(resolveServerCommand("noyalib-lsp", "linux"), { command: "noyalib-lsp" });
});

test("an empty setting falls back to the default", () => {
  assert.deepStrictEqual(resolveServerCommand("", "linux"), { command: "noyalib-lsp" });
  assert.deepStrictEqual(resolveServerCommand(undefined, "linux"), { command: "noyalib-lsp" });
});

test("an absolute path is used as given", () => {
  assert.deepStrictEqual(resolveServerCommand("/usr/local/bin/noyalib-lsp", "linux"), {
    command: "/usr/local/bin/noyalib-lsp",
  });
  assert.deepStrictEqual(resolveServerCommand("C:\\tools\\noyalib-lsp.exe", "win32"), {
    command: "C:\\tools\\noyalib-lsp.exe",
  });
});

test("a relative path is refused", () => {
  for (const [value, platform] of [
    ["./bin/noyalib-lsp", "linux"],
    ["tools/noyalib-lsp", "darwin"],
    ["..\\evil.exe", "win32"],
    ["C:evil.exe", "win32"],
  ]) {
    const got = resolveServerCommand(value, platform);
    assert.ok(got.error, `${value} on ${platform} should be refused, got ${JSON.stringify(got)}`);
    assert.strictEqual(got.command, undefined);
  }
});
