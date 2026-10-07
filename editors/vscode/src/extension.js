// SPDX-FileCopyrightText: 2026 Noyalib
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The VS Code client for noyalib-lsp. It does one thing: start the language
// server on the binary named by `noyalib.path` (default: `noyalib-lsp` on
// PATH) for YAML documents, and restart it on request. Diagnostics,
// formatting and hover all come from the server.
//
// What it runs is chosen with care, because opening a folder must not be
// enough to execute a program from it: `noyalib.path` is ignored in
// workspaces the user has not trusted (package.json declares it as a
// restricted configuration), a relative path is refused, and the server
// starts in the user's home directory rather than the workspace.
"use strict";

const os = require("os");
const vscode = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");
const { resolveServerCommand, DEFAULT_COMMAND } = require("./server-command");

let client;

function configuredCommand() {
  return resolveServerCommand(vscode.workspace.getConfiguration("noyalib").get("path", DEFAULT_COMMAND));
}

function serverOptions(command) {
  return { command, args: [], transport: TransportKind.stdio, options: { cwd: os.homedir() } };
}

async function offerSettings(message) {
  const choice = await vscode.window.showErrorMessage(message, "Open settings");
  if (choice === "Open settings") {
    vscode.commands.executeCommand("workbench.action.openSettings", "noyalib.path");
  }
}

async function start(context) {
  const resolved = configuredCommand();
  if (resolved.error) {
    client = undefined;
    await offerSettings(`noyalib: ${resolved.error}`);
    return;
  }
  client = new LanguageClient(
    "noyalib",
    "noyalib YAML",
    serverOptions(resolved.command),
    {
      documentSelector: [{ scheme: "file", language: "yaml" }, { scheme: "untitled", language: "yaml" }],
      synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher("**/*.{yaml,yml}") },
    },
  );
  context.subscriptions.push(client);
  try {
    await client.start();
  } catch (error) {
    await offerSettings(
      `noyalib: could not start "${resolved.command}". Install it with \`cargo install noyalib-lsp --locked\` or set noyalib.path.`,
    );
    throw error;
  }
}

async function restart(context) {
  if (client) {
    await client.stop();
  }
  await start(context);
}

async function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("noyalib.restart", async () => {
      await restart(context);
      vscode.window.setStatusBarMessage("noyalib: language server restarted", 3000);
    }),
  );
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration("noyalib.path")) {
        await restart(context);
      }
    }),
  );
  // Trusting the workspace makes its noyalib.path apply.
  context.subscriptions.push(vscode.workspace.onDidGrantWorkspaceTrust(() => restart(context)));
  await start(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
