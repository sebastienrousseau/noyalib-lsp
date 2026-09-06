// SPDX-FileCopyrightText: 2026 Noyalib
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The VS Code client for noyalib-lsp. It does one thing: start the language
// server on the binary named by `noyalib.path` (default: `noyalib-lsp` on
// PATH) for YAML documents, and restart it on request. Diagnostics,
// formatting and hover all come from the server.
"use strict";

const vscode = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;

function serverOptions() {
  const command = vscode.workspace.getConfiguration("noyalib").get("path", "noyalib-lsp");
  return { command, args: [], transport: TransportKind.stdio };
}

async function start(context) {
  client = new LanguageClient(
    "noyalib",
    "noyalib YAML",
    serverOptions(),
    {
      documentSelector: [{ scheme: "file", language: "yaml" }, { scheme: "untitled", language: "yaml" }],
      synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher("**/*.{yaml,yml}") },
    },
  );
  context.subscriptions.push(client);
  try {
    await client.start();
  } catch (error) {
    const path = serverOptions().command;
    const choice = await vscode.window.showErrorMessage(
      `noyalib: could not start "${path}". Install it with \`cargo install noyalib-lsp --locked\` or set noyalib.path.`,
      "Open settings",
    );
    if (choice === "Open settings") {
      vscode.commands.executeCommand("workbench.action.openSettings", "noyalib.path");
    }
    throw error;
  }
}

async function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("noyalib.restart", async () => {
      if (client) {
        await client.stop();
      }
      await start(context);
      vscode.window.setStatusBarMessage("noyalib: language server restarted", 3000);
    }),
  );
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration("noyalib.path") && client) {
        await client.stop();
        await start(context);
      }
    }),
  );
  await start(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
