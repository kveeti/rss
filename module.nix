{ rssPkg }:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.rss;
  dataDir = "/var/lib/rss";
in
{
  options.services.rss = {
    enable = lib.mkEnableOption "RSS Service";

    environment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = {};
      description = "Environment variables to pass to the service";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Runtime path to an environment file containing service configuration and secrets";
    };
  };

  config = lib.mkIf cfg.enable {
    users.groups.rss = { };
    users.users.rss = {
      isSystemUser = true;
      group = "rss";
    };

    systemd.services.rss = {
      description = "RSS Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      environment = cfg.environment;

      serviceConfig = {
        ExecStart = "${rssPkg}/bin/rss";
        Restart = "always";
        User = "rss";
        Group = "rss";

        StateDirectory = "rss";
        StateDirectoryMode = "0700";
        WorkingDirectory = dataDir;

        EnvironmentFile = lib.mkIf (cfg.environmentFile != null) cfg.environmentFile;

        CapabilityBoundingSet = [ "" ];
        DeviceAllow = [ "" ];
        LimitCORE = 0;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateMounts = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProcSubset = "pid";
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectProc = "invisible";
        ProtectSystem = "strict";
        RemoveIPC = true;
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [ "@system-service" "~@privileged" "~@resources" ];
        UMask = "0077";
      };
    };
  };
}
