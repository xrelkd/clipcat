{ rustToolchain
, cargoArgs
, unitTestArgs
, pkgs
, ...
}:

let
  cargo-ext = pkgs.callPackage ./cargo-ext.nix { inherit cargoArgs unitTestArgs; };
in
pkgs.mkShell {
  name = "dev-shell";

  nativeBuildInputs = with pkgs; [
    cargo-ext.cargo-build-all
    cargo-ext.cargo-clippy-all
    cargo-ext.cargo-doc-all
    cargo-ext.cargo-nextest-all
    cargo-ext.cargo-test-all
    cargo-nextest
    cargo-udeps
    cargo-watch
    rustToolchain

    tokei

    llvmPackages_16.clang
    llvmPackages_16.libclang

    pkg-config

    protobuf
    python3

    jq

    hclfmt
    nixpkgs-fmt
    nodePackages.prettier
    shfmt
    sleek
    taplo
    treefmt
    # clang-tools contains clang-format
    clang-tools

    shellcheck
  ];

  buildInputs = with pkgs; [
    xorg.libxcb
  ];

  PROTOC = "${pkgs.protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${pkgs.protobuf}/include";

  LIBCLANG_PATH = "${pkgs.llvmPackages_16.libclang.lib}/lib";

  shellHook = ''
    export NIX_PATH="nixpkgs=${pkgs.path}"
  '';
}
