{
  description = "clipcat is a clipboard manager written in Rust Programming Language.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; config.allowUnfree = true; };
        inherit (pkgs) callPackage;
      in
      {
        packages = rec {
          clipcat = callPackage ./default.nix { };
          default = clipcat;
        };
        devShells = rec {
          clipcat = callPackage ./shell.nix { };
          default = clipcat;
        };
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
