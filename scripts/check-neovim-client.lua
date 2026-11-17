-- SPDX-FileCopyrightText: 2026 Noyalib
-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Run with an isolated editor configuration:
-- NOYALIB_LSP_BIN=/absolute/path/to/noyalib-lsp nvim --headless -u NONE -i NONE -n -l scripts/check-neovim-client.lua

local function run()
  local binary = assert(vim.env.NOYALIB_LSP_BIN, 'set NOYALIB_LSP_BIN to the tested server')
  local buffer = vim.api.nvim_create_buf(true, false)
  vim.api.nvim_set_current_buf(buffer)
  vim.api.nvim_buf_set_name(buffer, vim.fn.tempname() .. '.yaml')
  vim.bo[buffer].filetype = 'yaml'
  local source = { '# keep', 'name: 🚀', 'port: 8080' }
  vim.api.nvim_buf_set_lines(buffer, 0, -1, false, source)
  local id = assert(vim.lsp.start({
    name = 'noyalib-client-check',
    cmd = { binary },
    root_dir = vim.fn.getcwd(),
    flags = { debounce_text_changes = 0 },
  }))
  assert(vim.wait(10000, function()
    local client = vim.lsp.get_client_by_id(id)
    return client and client.initialized and vim.lsp.buf_is_attached(buffer, id)
  end, 20), 'server initialization timed out')
  local client = assert(vim.lsp.get_client_by_id(id))
  assert(client.server_capabilities.hoverProvider, 'hover capability missing')
  assert(client.server_capabilities.documentFormattingProvider, 'format capability missing')
  assert(not client.server_capabilities.completionProvider, 'unsupported completion advertised')
  local reply = assert(client:request_sync('textDocument/hover', {
    textDocument = { uri = vim.uri_from_bufnr(buffer) },
    position = { line = 1, character = 8 },
  }, 5000, buffer))
  assert(not reply.err, vim.inspect(reply.err))
  assert(reply.result.contents.value:find('mapping', 1, true), 'document type hover missing')
  vim.api.nvim_buf_set_lines(buffer, 0, -1, false, { 'a: [' })
  assert(vim.wait(10000, function()
    return #vim.diagnostic.get(buffer) > 0
  end, 20), 'invalid change did not publish diagnostics')
  vim.api.nvim_buf_set_lines(buffer, 0, -1, false, source)
  assert(vim.wait(10000, function()
    return #vim.diagnostic.get(buffer) == 0
  end, 20), 'valid change did not clear diagnostics')
  vim.lsp.buf.format({ bufnr = buffer, async = false, timeout_ms = 5000 })
  assert(vim.api.nvim_buf_get_lines(buffer, 0, 1, false)[1] == '# keep', 'formatter lost comment')
  client:stop()
  assert(vim.wait(5000, function() return client:is_stopped() end, 20), 'shutdown timed out')
  print(vim.json.encode({ status = 'passed', client = 'Neovim', version = vim.version(),
    checks = { 'initialize', 'capabilities', 'unicode hover', 'didChange diagnostics',
      'clear diagnostics', 'format comments', 'shutdown' } }))
end

local ok, error = pcall(run)
if not ok then
  io.stderr:write(tostring(error) .. '\n')
  vim.cmd('cquit 1')
else
  vim.cmd('qa!')
end
