{ name
, version
, lib
, rustPlatform
, llvmPackages
, protobuf
, pkg-config
, xvfb-run
, cargo-nextest
, installShellFiles
}:

rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

  src = lib.cleanSource ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  nativeBuildInputs = [
    llvmPackages.clang
    llvmPackages.libclang

    pkg-config

    protobuf

    installShellFiles
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

  postInstall = ''
    for cmd in clipcatd clipcatctl clipcat-menu clipcat-notify; do
      installShellCompletion --cmd $cmd \
        --bash <($out/bin/$cmd completions bash) \
        --fish <($out/bin/$cmd completion fish) \
        --zsh  <($out/bin/$cmd completion zsh)
    done
  '';

  meta = with lib; {
    description = "Clipboard Manager written in Rust Programming Language";
    homepage = "https://github.com/xrelkd/clipcat";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    maintainers = with maintainers; [ xrelkd ];
    mainProgram = "clipcatd";
  };
}
