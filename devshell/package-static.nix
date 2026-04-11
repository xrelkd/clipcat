{
  name,
  version,
  lib,
  rustPlatform,
  installShellFiles,
  protobuf,
  completions ? null,
}:

rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

  src = lib.cleanSource ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    installShellFiles
    protobuf
  ];

  doCheck = false;

  postInstall =
    if completions != null then
      ''
        mkdir -p $out/share
        cp -r ${completions}/share/* $out/share/
      ''
    else
      ''
        for cmd in clipcatd clipcatctl clipcat-menu clipcat-notify; do
          installShellCompletion --cmd $cmd \
            --bash <($out/bin/$cmd completions bash) \
            --fish <($out/bin/$cmd completions fish) \
            --zsh  <($out/bin/$cmd completions zsh)
        done
      '';

  meta = with lib; {
    description = "Clipboard Manager written in Rust Programming Language (statically linked)";
    homepage = "https://github.com/xrelkd/clipcat";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    maintainers = with maintainers; [ xrelkd ];
    mainProgram = "clipcatd";
  };
}
