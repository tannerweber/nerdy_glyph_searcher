{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-26.05/nixexprs.tar.zst";
  };

  outputs = inputs: {
    devShells = builtins.mapAttrs (system: pkgs: {
      default = pkgs.callPackage ./shell.nix { };
    }) inputs.nixpkgs.legacyPackages;
    packages = builtins.mapAttrs (system: pkgs: {
      default = pkgs.callPackage ./default.nix { };
    }) inputs.nixpkgs.legacyPackages;
  };
}
