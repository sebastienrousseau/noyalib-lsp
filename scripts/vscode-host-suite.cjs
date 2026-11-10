// SPDX-FileCopyrightText: 2026 Noyalib
// SPDX-License-Identifier: MIT OR Apache-2.0

'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { setTimeout: delay } = require('node:timers/promises');
const vscode = require('vscode');

async function until(predicate, message) {
  const end = Date.now() + 12000;
  do {
    const value = await predicate();
    if (value) return value;
    await delay(100);
  } while (Date.now() < end);
  throw new Error(message);
}

exports.run = async () => {
  const extension = vscode.extensions.getExtension('sebastienrousseau.noyalib');
  assert(extension, 'VSIX not installed in the isolated profile');
  const installed = path.relative(process.env.NOYALIB_HOST_EXTENSIONS, extension.extensionPath);
  assert(installed && !installed.startsWith('..') && !path.isAbsolute(installed), extension.extensionPath);
  assert.equal(vscode.workspace.getConfiguration('noyalib').get('path'), process.env.NOYALIB_HOST_SERVER);

  const file = path.join(process.env.NOYALIB_HOST_WORKSPACE, 'document.yaml');
  fs.writeFileSync(file, 'name: [\n');
  const document = await vscode.workspace.openTextDocument(file);
  assert.equal(document.languageId, 'yaml');
  await vscode.window.showTextDocument(document);
  await extension.activate();
  const diagnostics = () => vscode.languages.getDiagnostics(document.uri);
  await until(() => diagnostics().some(item => item.severity === vscode.DiagnosticSeverity.Error),
    'installed client did not publish parse diagnostics');

  const source = '# keep\nname: 🚀\nport:    8080\n';
  const update = new vscode.WorkspaceEdit();
  update.replace(document.uri, new vscode.Range(document.positionAt(0),
    document.positionAt(document.getText().length)), source);
  assert(await vscode.workspace.applyEdit(update));
  await until(() => diagnostics().length === 0, 'fixed buffer did not clear diagnostics');

  const hover = await vscode.commands.executeCommand('vscode.executeHoverProvider', document.uri,
    new vscode.Position(1, 8));
  assert(hover.some(item => item.contents.some(content =>
    (typeof content === 'string' ? content : content.value).includes('mapping'))), 'document hover missing');
  const edits = await vscode.commands.executeCommand('vscode.executeFormatDocumentProvider', document.uri,
    { tabSize: 2, insertSpaces: true });
  assert(edits.length > 0, 'formatter returned no edits for non-canonical input');
  const format = new vscode.WorkspaceEdit();
  format.set(document.uri, edits);
  assert(await vscode.workspace.applyEdit(format));
  assert.equal(document.getText(), '# keep\nname: 🚀\nport: 8080\n');
  await vscode.commands.executeCommand('noyalib.restart');
  await until(async () => {
    const result = await vscode.commands.executeCommand('vscode.executeHoverProvider', document.uri,
      new vscode.Position(1, 8));
    return result.length > 0;
  }, 'restarted client did not answer hover');

  fs.writeFileSync(process.env.NOYALIB_HOST_REPORT, JSON.stringify({
    status: 'passed', editor: vscode.version, extension: extension.packageJSON.version,
    checks: ['VSIX install', 'configured binary', 'diagnostics', 'didChange clear', 'Unicode hover',
      'comment-preserving format', 'server restart'],
  }));
};
