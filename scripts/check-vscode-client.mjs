// SPDX-FileCopyrightText: 2026 Noyalib
// SPDX-License-Identifier: MIT OR Apache-2.0

// Launch an isolated VS Code profile with the VSIX actually installed.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const [vsixArg, serverArg] = process.argv.slice(2);
const cli = process.env.VS_CODE_CLI;
const editor = process.env.VS_CODE_BIN || cli;
assert(vsixArg && serverArg && cli, 'set VS_CODE_CLI and run with <vsix> <noyalib-lsp>; optionally set VS_CODE_BIN');
const vsix = resolve(vsixArg);
const server = resolve(serverArg);
const suite = fileURLToPath(new URL('./vscode-host-suite.cjs', import.meta.url));
const hash = file => createHash('sha256').update(readFileSync(file)).digest('hex');
const scratch = mkdtempSync(join(tmpdir(), 'noyalib-vscode-host-'));
try {
  const data = join(scratch, 'profile');
  const extensions = join(scratch, 'extensions');
  const harness = join(scratch, 'harness');
  const workspace = join(scratch, 'workspace');
  const report = join(scratch, 'receipt.json');
  mkdirSync(join(data, 'User'), { recursive: true });
  mkdirSync(extensions);
  mkdirSync(harness);
  mkdirSync(workspace);
  writeFileSync(join(data, 'User/settings.json'), JSON.stringify({
    'noyalib.path': server,
    'telemetry.telemetryLevel': 'off',
    'update.mode': 'none',
    'extensions.autoCheckUpdates': false,
    'extensions.autoUpdate': false,
    'security.workspace.trust.enabled': false,
  }));
  writeFileSync(join(harness, 'package.json'), JSON.stringify({
    name: 'noyalib-host-harness', version: '1.0.0', engines: { vscode: '^1.91.0' },
  }));
  const env = { ...process.env, NOYALIB_HOST_REPORT: report, NOYALIB_HOST_WORKSPACE: workspace,
    NOYALIB_HOST_SERVER: server, NOYALIB_HOST_EXTENSIONS: extensions };
  delete env.VSCODE_IPC_HOOK_CLI;
  const run = (command, args, timeout) => execFileSync(command, args, {
    env, cwd: scratch, timeout, encoding: 'utf8', stdio: ['ignore', 'inherit', 'inherit'],
  });
  const flags = ['--user-data-dir', data, '--extensions-dir', extensions];
  run(cli, [...flags, '--install-extension', vsix], 60000);
  run(editor, [...flags, '--skip-welcome', '--skip-release-notes', '--disable-updates',
    '--disable-extension', 'github.copilot-chat', '--disable-extension', 'github.copilot',
    '--extensionDevelopmentPath=' + harness, '--extensionTestsPath=' + suite, workspace], 75000);
  const result = JSON.parse(readFileSync(report, 'utf8'));
  assert.equal(result.status, 'passed');
  console.log(JSON.stringify({ ...result, vsix_sha256: hash(vsix), server_sha256: hash(server) }, null, 2));
} finally {
  rmSync(scratch, { recursive: true, force: true, maxRetries: 3 });
}
