{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/21.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      supportedSystems = [
        flake-utils.lib.system.aarch64-darwin
        flake-utils.lib.system.x86_64-darwin
      ];
    in
      flake-utils.lib.eachSystem supportedSystems (
        system: let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
          rec {
            # `nix develop`
            devShell = pkgs.mkShell {
              buildInputs = [
                pkgs.nodejs
              ];
            };
          }
      );
}
