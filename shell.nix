with import <nixpkgs> { };

stdenv.mkDerivation {
  name = "clipcat-dev";

  RUST_BACKTRACE = 1;

  LIBCLANG_PATH = "${llvmPackages.libclang}/lib";

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  nativeBuildInputs = [
    rustup
    cargo-make

    clang
    llvmPackages.libclang

    pkgconfig

    protobuf
    python3
  ];

  buildInputs = [ xorg.libxcb ];
}
