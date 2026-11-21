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

  buildInputs = lib.optionals stdenv.hostPlatform.isDarwin [
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
      nixfmt
      prettier
      shfmt
      taplo
      treefmt

      shellcheck

      pkg-config

      typos
    ]
    ++ lib.optionals stdenv.hostPlatform.isLinux [
      xvfb-run
    ];

  shellHook = ''
    export NIX_PATH="nixpkgs=${pkgs.path}"

    mkdir -p .cargo
    if [ ! -f .cargo/config.toml ] || ! grep -q 'x86_64-unknown-linux-musl' .cargo/config.toml 2>/dev/null; then
      cat >> .cargo/config.toml << EOF
    [target.x86_64-unknown-linux-musl]
    linker = "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc"
    EOF
    fi
  '';
}
