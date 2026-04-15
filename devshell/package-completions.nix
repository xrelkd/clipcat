{ pkgs, clipcat }:

pkgs.runCommand "clipcat-completions"
  {
    nativeBuildInputs = [ pkgs.installShellFiles ];
  }

  ''
    for cmd in clipcatd clipcatctl clipcat-menu clipcat-notify; do
      installShellCompletion --cmd $cmd \
        --bash <(${clipcat}/bin/$cmd completions bash) \
        --fish <(${clipcat}/bin/$cmd completions fish) \
        --zsh  <(${clipcat}/bin/$cmd completions zsh)
    done
  ''
