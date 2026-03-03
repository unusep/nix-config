{ ... }: {
  launchd.agents.hrm = {
    enable = true;
    config = {
      Label = "com.wontaeyang.HRM.launcher";
      ProgramArguments = [ "open" "-a" "/Applications/HRM.app" ];
      RunAtLoad = true;
      StandardOutPath = "/tmp/hrm-launcher.log";
      StandardErrorPath = "/tmp/hrm-launcher.log";
    };
  };

  home.file."Library/Application Support/HRM/config.json" = {
    force = true;
    text = builtins.toJSON {
      enabled = true;
      quickTapTermMs = 150;
      requirePriorIdleMs = 150;
      bilateralFiltering = true;
      holdTriggerOnRelease = false;
      keyBindings = [
        {
          keyCode = 0;
          label = "A";
          modifier = "control";
          enabled = true;
          position = "leftPinky";
        }
        {
          keyCode = 1;
          label = "S";
          modifier = "option";
          enabled = true;
          position = "leftRing";
        }
        {
          keyCode = 2;
          label = "D";
          modifier = "command";
          enabled = true;
          position = "leftMiddle";
        }
        {
          keyCode = 3;
          label = "F";
          modifier = "shift";
          enabled = true;
          position = "leftIndex";
        }
        {
          keyCode = 5;
          label = "G";
          enabled = false;
          position = "leftIndexInner";
        }
        {
          keyCode = 4;
          label = "H";
          enabled = false;
          position = "rightIndexInner";
        }
        {
          keyCode = 38;
          label = "J";
          modifier = "shift";
          enabled = true;
          position = "rightIndex";
        }
        {
          keyCode = 40;
          label = "K";
          modifier = "command";
          enabled = true;
          position = "rightMiddle";
        }
        {
          keyCode = 37;
          label = "L";
          modifier = "option";
          enabled = true;
          position = "rightRing";
        }
        {
          keyCode = 41;
          label = ";";
          modifier = "control";
          enabled = true;
          position = "rightPinky";
        }
      ];
    };
  };
}
