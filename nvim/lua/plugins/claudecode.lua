-- Configure Claude Code terminal width
return {
  {
    "anthropics/claudecode.nvim",
    opts = {
      terminal = {
        split_width_percentage = 0.40, -- 40% of screen (default is 0.30)
        split_side = "right", -- "left" or "right"
      },
    },
  },
}
