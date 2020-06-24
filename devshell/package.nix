{ name
, version
, lib
, rustPlatform
  # , llvmPackages_15
  # , protobuf
  # , pkg-config
  # , openssl
}:

rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

  doCheck = false;

  src = lib.cleanSource ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  buildInputs = [
    # openssl
  ];

  nativeBuildInputs = [
    # pkg-config

    # llvmPackages_15.clang
    # llvmPackages_15.libclang
  ];

  # PROTOC = "${protobuf}/bin/protoc";
  # PROTOC_INCLUDE = "${protobuf}/include";
  #
  # LIBCLANG_PATH = "${llvmPackages_15.libclang.lib}/lib";
}
