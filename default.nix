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

  cargoSha256 = "sha256-ZHZeM69iFOTPpqhOGENxrXNnO921s2E5Shcazobgizs=";

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

  buildFeatures = [ "all-bins" ];

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
