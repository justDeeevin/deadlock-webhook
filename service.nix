{
  lib,
  config,
  ...
}:
{
  _class = "service";
  config = {
    process.argv = [ (lib.getExe config.deadlock-webhook) ];
    systemd.service.serviceConfig = {
      Type = "oneshot";
      StateDirectory = "deadlock-webhook";
    };
    systemd.timers.deadlock-webhook = {
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnBootSec = "5min";
        OnUnitActiveSec = "4h";
        Persistent = true;

        Unit = "deadlock-webhook.service";
      };
    };
  };
  options.deadlock-webhook = {
    package = lib.mkOption {
      type = lib.types.package;
      description = "The deadlock-webhook package to use";
      defaultText = "The package that provided this service";
    };
    webhook_url = lib.mkOption {
      type = lib.types.str;
      description = "URL of the Discord webhook to use";
    };
    role_id = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      description = "ID of the role to mention in the message (if unset, the program will ping everyone)";
      default = null;
    };
  };
}
