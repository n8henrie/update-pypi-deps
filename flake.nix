{
  description = "Flake for https://github.com/n8henrie/update-pypi-deps";

  inputs.nixpkgs.url = "github:nixos/nixpkgs";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      eachSystem =
        with nixpkgs.lib;
        f: foldAttrs mergeAttrs { } (map (s: mapAttrs (_: v: { ${s} = v; }) (f s)) systems);
      inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) name;
    in
    {
      overlays = {
        default = self.overlays.${name};
        ${name} = _: prev: {
          # inherit doesn't work with dynamic attributes
          ${name} = self.packages.${prev.system}.${name};
        };
      };
    }
    // (eachSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) name;
      in
      {

        packages = {
          default = self.packages.${system}.${name};
          ${name} = pkgs.callPackage ./. { };
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.${name}}/bin/${name}";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-analyzer
            cargo
          ];
        };
      }
    ));
}
