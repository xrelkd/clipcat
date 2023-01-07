{
  description = "clipcat is a clipboard manager written in Rust Programming Language.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; config.allowUnfree = true; };
      in rec {
        packages = rec {
          clipcat = pkgs.callPackage ./default.nix { };
          default = clipcat;
        };
        devShells = rec {
          clipcat = import ./shell.nix { inherit (packages) clipcat; inherit (pkgs) mkShell; };
          default = clipcat;
        };
      }
    );
}
