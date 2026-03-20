-- Autocmds are automatically loaded on the VeryLazy event
-- Default autocmds that are always set: https://github.com/LazyVim/LazyVim/blob/main/lua/lazyvim/config/autocmds.lua
--
-- Add any additional autocmds here
-- with `vim.api.nvim_create_autocmd`
--
-- Or remove existing autocmds by their group name (which is prefixed with `lazyvim_` for the defaults)
-- e.g. vim.api.nvim_del_augroup_by_name("lazyvim_wrap_spell")

-- Check for external file changes when focus returns
vim.api.nvim_create_autocmd({ "FocusGained", "TermClose" }, {
  command = "checktime",
})

-- Also check when entering a buffer (catches Claude Code diffs)
vim.api.nvim_create_autocmd("BufEnter", {
  command = "checktime",
})

-- Check when leaving insert mode (catches Claude Code changes)
vim.api.nvim_create_autocmd("InsertLeave", {
  command = "checktime",
  group = vim.api.nvim_create_augroup("file_change_check", { clear = true }),
})

vim.api.nvim_del_augroup_by_name("lazyvim_wrap_spell")
