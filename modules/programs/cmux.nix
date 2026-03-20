{ lib, ... }:
let
  domain = "com.cmuxterm.app";

  boolStr = b: if b then "true" else "false";

  mkShortcut = { key, cmd ? false, shift ? false, option ? false, ctrl ? false }:
    ''{"key":"${key}","command":${boolStr cmd},"shift":${boolStr shift},"option":${boolStr option},"control":${boolStr ctrl}}'';

  keybindings = {
    "shortcut.toggleSidebar"     = mkShortcut { key = "s"; cmd = true; };
    "shortcut.newTab"            = mkShortcut { key = "n"; cmd = true; };
    "shortcut.newWindow"         = mkShortcut { key = "n"; cmd = true; shift = true; };
    "shortcut.closeWindow"       = mkShortcut { key = "w"; cmd = true; ctrl = true; };
    "shortcut.openFolder"        = mkShortcut { key = "o"; cmd = true; };
    "shortcut.showNotifications" = mkShortcut { key = "i"; cmd = true; };
    "shortcut.jumpToUnread"      = mkShortcut { key = "u"; cmd = true; shift = true; };

    "shortcut.nextSurface"    = mkShortcut { key = "l"; cmd = true; };
    "shortcut.prevSurface"    = mkShortcut { key = "h"; cmd = true; };
    "shortcut.triggerFlash"   = mkShortcut { key = "h"; cmd = true; shift = true; ctrl = true; option = true; };
    "shortcut.nextSidebarTab" = mkShortcut { key = "j"; cmd = true; };
    "shortcut.prevSidebarTab" = mkShortcut { key = "k"; cmd = true; };
    "shortcut.renameTab"      = mkShortcut { key = "r"; cmd = true; };
    "shortcut.renameWorkspace" = mkShortcut { key = "r"; cmd = true; shift = true; };
    "shortcut.closeWorkspace" = mkShortcut { key = "w"; cmd = true; shift = true; };
    "shortcut.newSurface"     = mkShortcut { key = "t"; cmd = true; };

    "shortcut.focusLeft"       = mkShortcut { key = "h"; cmd = true; shift = true; };
    "shortcut.focusRight"      = mkShortcut { key = "l"; cmd = true; shift = true; };
    "shortcut.focusUp"         = mkShortcut { key = "k"; cmd = true; shift = true; };
    "shortcut.focusDown"       = mkShortcut { key = "j"; cmd = true; shift = true; };
    "shortcut.splitRight"      = mkShortcut { key = "d"; cmd = true; };
    "shortcut.splitDown"       = mkShortcut { key = "d"; cmd = true; shift = true; };
    "shortcut.toggleSplitZoom" = mkShortcut { key = "\\r"; cmd = true; shift = true; };

    "shortcut.splitBrowserRight" = mkShortcut { key = "d"; cmd = true; option = true; };
    "shortcut.splitBrowserDown"  = mkShortcut { key = "d"; cmd = true; shift = true; option = true; };
    "shortcut.openBrowser"       = mkShortcut { key = "b"; cmd = true; };
    "shortcut.toggleBrowserDeveloperTools"  = mkShortcut { key = "i"; cmd = true; option = true; };
    "shortcut.showBrowserJavaScriptConsole" = mkShortcut { key = "c"; cmd = true; option = true; };
  };

  writeShortcut = name: json:
    ''$DRY_RUN_CMD /usr/bin/defaults write ${domain} "${name}" -data "$(printf '%s' '${json}' | /usr/bin/xxd -p | /usr/bin/tr -d '\n')"'';

  writeString = name: value:
    ''$DRY_RUN_CMD /usr/bin/defaults write ${domain} "${name}" -string "${value}"'';

  writeInt = name: value:
    ''$DRY_RUN_CMD /usr/bin/defaults write ${domain} "${name}" -int ${toString value}'';
in {
  home.activation.cmux-defaults = lib.hm.dag.entryAfter ["writeBoundary"] ''
    ${lib.concatStringsSep "\n    " (lib.mapAttrsToList writeShortcut keybindings)}

    ${writeString "appearanceMode" "system"}
    ${writeString "browserThemeMode" "system"}
    ${writeInt "warnBeforeQuitShortcut" 0}
    ${writeString "sidebarPreset" "nativeSidebar"}
    ${writeString "sidebarBlendMode" "withinWindow"}
    ${writeInt "sidebarBlurOpacity" 1}
    ${writeInt "sidebarCornerRadius" 0}
    ${writeString "sidebarMaterial" "sidebar"}
    ${writeString "sidebarState" "followWindow"}
    ${writeString "sidebarTintHex" "#000000"}
    ${writeString "sidebarTintOpacity" "0.18"}
    ${writeString "sidebarActiveTabIndicatorStyle" "solidFill"}
    ${writeInt "sidebarAppearanceDefaultsVersion" 1}
    ${writeInt "sidebarShowBranchDirectory" 0}
    ${writeInt "sidebarShowLog" 1}
    ${writeInt "sidebarShowPorts" 1}
    ${writeInt "sidebarShowProgress" 1}
    ${writeInt "sidebarShowPullRequest" 0}
    ${writeInt "sidebarShowStatusPills" 1}

    ${writeString "socketControlMode" "automation"}
    ${writeString "newWorkspacePlacement" "end"}
    ${writeInt "browserOpenSidebarPullRequestLinksInCmuxBrowser" 0}
    ${writeInt "browserOpenTerminalLinksInCmuxBrowser" 0}
    ${writeInt "workspaceAutoReorderOnNotification" 0}
  '';
}
