// SPDX-FileCopyrightText: 2026 Noyalib
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The settings that choose what the extension executes must not be
// taken from a workspace the user has not trusted.
"use strict";

const test = require("node:test");
const assert = require("node:assert");
const manifest = require("../package.json");

test("noyalib.path is a machine setting a workspace cannot set silently", () => {
  const setting = manifest.contributes.configuration.properties["noyalib.path"];
  assert.strictEqual(setting.scope, "machine-overridable");
});

test("untrusted workspaces run in limited mode without their noyalib.path", () => {
  const untrusted = manifest.capabilities && manifest.capabilities.untrustedWorkspaces;
  assert.ok(untrusted, "capabilities.untrustedWorkspaces is declared");
  assert.strictEqual(untrusted.supported, "limited");
  assert.deepStrictEqual(untrusted.restrictedConfigurations, ["noyalib.path"]);
});
