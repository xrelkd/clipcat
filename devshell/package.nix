{ name
, version
, lib
, rustPlatform
, llvmPackages
, protobuf
, pkg-config
}:

rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

  doCheck = false;

  src = lib.cleanSource ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    llvmPackages.clang
    llvmPackages.libclang

    pkg-config

    protobuf
  ];

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
}
