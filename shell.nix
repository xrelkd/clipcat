{ pkgs ? (import <nixpkgs> {}) }:
let
  inherit (pkgs) callPackage mkShell clippy cargo;
  clipcat = callPackage ./default.nix { };
in
mkShell {
  inputsFrom = [ clipcat ];
  buildInputs = [
    clippy
  ];
  # needed for internal protobuf c wrapper library
  inherit (clipcat)
    PROTOC
    PROTOC_INCLUDE;
}
