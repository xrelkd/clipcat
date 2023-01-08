{ lib
, installShellFiles
, rustPlatform
, rustfmt
, xorg
, pkg-config
, llvmPackages
, clang
, protobuf
, python3
}:

rustPlatform.buildRustPackage {
  pname = "clipcat";
  version = "0.5.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "x11-clipboard-0.7.0" = "sha256-E/3R94DB3gHhK2mnEc1UF/i0FvbRHi4uDFmRZsAtXS0=";
    };
  };

  # needed for internal protobuf c wrapper library
  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  nativeBuildInputs = [
    pkg-config

    rustPlatform.bindgenHook

    rustfmt
    protobuf

    python3

    installShellFiles
  ];
  buildInputs = [ xorg.libxcb ];

  buildFeatures = [ "all-bins,x11" ];

  postInstall = ''
    installShellCompletion --bash completions/bash-completion/completions/*
    installShellCompletion --fish completions/fish/completions/*
    installShellCompletion --zsh  completions/zsh/site-functions/*
  '';

  meta = with lib; {
    description = "Clipboard Manager written in Rust Programming Language";
    homepage = "https://github.com/xrelkd/clipcat";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    maintainers = with maintainers; [ xrelkd ];
  };
}
