{ ... }: {
  home.file.".config/ghostty/config" = {
    force = true;
    text = ''
      font-family = JetBrains Mono
      font-size = 13

      keybind = cmd+shift+s=activate_key_table:vim
      keybind = vim/j=scroll_page_lines:1
      keybind = vim/k=scroll_page_lines:-1
      keybind = vim/d=scroll_page_down
      keybind = vim/u=scroll_page_up
      keybind = vim/ctrl+f=scroll_page_down
      keybind = vim/ctrl+b=scroll_page_up
      keybind = vim/shift+j=scroll_page_down
      keybind = vim/shift+k=scroll_page_up
      keybind = vim/g>g=scroll_to_top
      keybind = vim/shift+g=scroll_to_bottom
      keybind = vim/slash=start_search
      keybind = vim/n=navigate_search:next
      keybind = vim/v=copy_to_clipboard
      keybind = vim/y=copy_to_clipboard
      keybind = vim/shift+semicolon=toggle_command_palette
      keybind = vim/escape=deactivate_key_table
      keybind = vim/q=deactivate_key_table
      keybind = vim/i=deactivate_key_table
      keybind = vim/catch_all=ignore
    '';
  };
}
