{ pkgs ? import ./nix { } }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    git
    rustup
    cargo-make

    clang
    llvmPackages.libclang

    pkgconfig

    protobuf
    python3
  ];

  buildInputs = with pkgs; [ xorg.libxcb ];

  RUST_BACKTRACE = 1;

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang}/lib";

  PROTOC = "${pkgs.protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${pkgs.protobuf}/include";
}
