{
  config,
  lib,
  pkgs,
  namespace,
  ...
}: let
  cfg = config.${namespace}.pgpilot;
in {
  options.${namespace}.pgpilot = {
    enable = lib.mkEnableOption "pgpilot PGP key manager";
  };

  config = lib.mkIf cfg.enable {
    home.packages = [pkgs.${namespace}.pgpilot];
  };
}
