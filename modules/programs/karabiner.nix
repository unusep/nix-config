{ ... }: {
  home.file.".config/karabiner/karabiner.json" = {
    force = true;
    text = builtins.toJSON {
      global = {
        check_for_updates_on_startup = true;
        show_in_menu_bar = true;
        show_profile_name_in_menu_bar = false;
      };
      profiles = [
        {
          complex_modifications = {
            parameters = {
              "basic.to_if_alone_timeout_milliseconds" = 200;
              "basic.to_if_held_down_threshold_milliseconds" = 200;
            };
            rules = [
              {
                description = "Change caps_lock to delete_or_backspace (with all modifiers)";
                manipulators = [
                  {
                    from = {
                      key_code = "caps_lock";
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["shift"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["shift"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["command"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["command"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["control"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["control"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["option"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["option"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["command" "shift"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["command" "shift"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["control" "shift"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["control" "shift"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["option" "shift"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["option" "shift"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["command" "option"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["command" "option"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["control" "option"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["control" "option"];
                      }
                    ];
                    type = "basic";
                  }
                  {
                    from = {
                      key_code = "caps_lock";
                      modifiers = {
                        mandatory = ["command" "control"];
                      };
                    };
                    to = [
                      {
                        key_code = "delete_or_backspace";
                        modifiers = ["command" "control"];
                      }
                    ];
                    type = "basic";
                  }
                ];
              }
            ];
          };
          devices = [];
          fn_function_keys = [];
          name = "Default profile";
          selected = true;
          simple_modifications = [];
          virtual_hid_keyboard = {
            country_code = 0;
            keyboard_type_v2 = "ansi";
          };
        }
      ];
    };
    onChange = ''
      /bin/launchctl kickstart -k gui/`id -u`/org.pqrs.service.agent.karabiner_console_user_server
    '';
  };
}
