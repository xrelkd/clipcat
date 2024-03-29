{ name
, version
, lib
, rustPlatform
, protobuf
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

  nativeBuildInputs = [
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
    cargo nextest run --release --workspace --no-fail-fast --no-capture
    EOF

    chmod +x test-runner
    xvfb-run --auto-servernum ./test-runner
  '';

  postInstall = ''
    for cmd in clipcatd clipcatctl clipcat-menu clipcat-notify; do
      installShellCompletion --cmd $cmd \
        --bash <($out/bin/$cmd completions bash) \
        --fish <($out/bin/$cmd completions fish) \
        --zsh  <($out/bin/$cmd completions zsh)
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
