with import <nixpkgs> { };

stdenv.mkDerivation {
  name = "env";

  LIBCLANG_PATH = "${llvmPackages.libclang}/lib";

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  buildInputs = [
    xorg.libxcb

    rustup
    rustfmt

    pkgconfig

    clang
    llvmPackages.libclang

    protobuf
    python3
  ];
}
