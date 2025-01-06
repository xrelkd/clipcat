{
  rustToolchain,
  cargoArgs,
  unitTestArgs,
  pkgs,
  lib,
  stdenv,
  darwin,
  ...
}:

let
  cargo-ext = pkgs.callPackage ./cargo-ext.nix { inherit cargoArgs unitTestArgs; };
in
pkgs.mkShell {
  name = "dev-shell";

  buildInputs = lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Cocoa
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  nativeBuildInputs =
    with pkgs;
    [
      cargo-ext.cargo-build-all
      cargo-ext.cargo-clippy-all
      cargo-ext.cargo-doc-all
      cargo-ext.cargo-nextest-all
      cargo-ext.cargo-test-all
      cargo-nextest
      rustToolchain

      tokei

      protobuf

      jq

      buf
      hclfmt
      nixfmt-rfc-style
      nodePackages.prettier
      shfmt
      taplo
      treefmt

      shellcheck

      pkg-config
      libgit2
    ]
    ++ lib.optionals stdenv.isLinux [
      xvfb-run
    ];

  shellHook = ''
    export NIX_PATH="nixpkgs=${pkgs.path}"
  '';
}
