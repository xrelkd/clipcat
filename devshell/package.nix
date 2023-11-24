{ name
, version
, lib
, rustPlatform
, llvmPackages
, protobuf
, pkg-config
, xvfb-run
, cargo-nextest
}:

rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

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

  nativeCheckInputs = [
    cargo-nextest

    xvfb-run
  ];

  checkPhase = ''
    cat >test-runner <<EOF
    #!/bin/sh
    export NEXTEST_RETRIES=5

    cargo --version
    rustc --version
    cargo nextest --version
    cargo nextest run --workspace --no-fail-fast --no-capture
    EOF

    chmod +x test-runner
    xvfb-run --auto-servernum ./test-runner
  '';

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
}
