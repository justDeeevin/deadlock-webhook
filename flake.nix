{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
        };

        deadlock-webhook = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
            passthru.services.default = {
              imports = [ ./service.nix ];
              deadlock-webhook.package = deadlock-webhook;
            };
          }
        );
      in
      {
        checks = {
          inherit deadlock-webhook;
        };

        packages.default = deadlock-webhook;

        apps.default = flake-utils.lib.mkApp {
          drv = deadlock-webhook;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
        };
      }
    )
    // {
      nixosModules.default =
        {
          pkgs,
          lib,
          config,
          ...
        }:
        {
          options.services.deadlock-webhook = {
            enable = lib.mkEnableOption "deadlock patch notes discord webhook notifier";
            package = lib.mkOption {
              type = lib.types.package;
              description = "The package to run.";
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
              defaultText = "this flake's package";
            };
            period = lib.mkOption {
              type = lib.types.str;
              description = "The period between checks.";
              default = "5min";
            };
            role_id = lib.mkOption {
              type = lib.types.nullOr lib.types.int;
              description = "The role id to mention. If unset, the webhook will ping everyone.";
              default = null;
            };
            webhook_url_file = lib.mkOption {
              type = lib.types.nullOr lib.types.path;
              description = "The file containing the webhook url.";
              default = null;
            };
            webhook_url = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              description = "The webhook url. Setting this manually is insecure; prefer using webhook_url_file";
              default = null;
            };
          };
          config =
            let
              cfg = config.services.deadlock-webhook;
            in
            lib.mkIf cfg.enable {
              assertions = [
                {
                  assertion = cfg.webhook_url != null || cfg.webhook_url_file != null;
                  message = "Either webhook_url or webhook_url_file must be set.";
                }
              ];
              systemd.services.deadlock-webhook = {
                environment = {
                  ROLE_ID = cfg.role_id;
                  WEBHOOK_URL = lib.mkIf (cfg.webhook_url != null) cfg.webhook_url;
                  WEBHOOK_URL_FILE = lib.mkIf (cfg.webhook_url_file != null) cfg.webhook_url_file;
                };
                serviceConfig = {
                  ExecStart = lib.getExe cfg.package;
                  Type = "oneshot";
                  StateDirectory = "deadlock-webhook";
                };
              };
              systemd.timers.deadlock-webhook = {
                wantedBy = [ "timers.target" ];
                timerConfig = {
                  OnBootSec = "5min";
                  OnUnitActiveSec = "5min";
                  Persistent = true;

                  Unit = "deadlock-webhook.service";
                };
              };
            };
        };
    };
}
