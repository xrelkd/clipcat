{
  name,
  version,
  lib,
  nfpm,
  clipcat-static,
  pkgs,
  stdenv,
  packager ? "deb",
  arch ? "amd64",
}:

let
  nfpmConfig = pkgs.replaceVars ./nfpm.yaml {
    NAME = name;
    VERSION = version;
    ARCH = arch;
  };
in
stdenv.mkDerivation {
  pname = "${name}-${packager}";
  inherit version;

  nativeBuildInputs = [ nfpm ];

  dontUnpack = true;
  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    runHook preInstall

    staging=$(mktemp -d)
    mkdir -p "$staging/usr/bin"
    mkdir -p "$staging/usr/share/bash-completion/completions"
    mkdir -p "$staging/usr/share/fish/vendor_completions.d"
    mkdir -p "$staging/usr/share/zsh/site-functions"

    for bin in ${clipcat-static}/bin/*; do
      binname=$(basename "$bin")
      cp "$bin" "$staging/usr/bin/$binname"
    done

    for f in ${clipcat-static}/share/bash-completion/completions/*; do
      base=$(basename "$f")
      case "$base" in
        clipcatd)
          cp "$f" "$staging/usr/share/bash-completion/completions/clipcatd.bash"
          ;;
        clipcatctl)
          cp "$f" "$staging/usr/share/bash-completion/completions/clipcatctl.bash"
          ;;
        clipcat-menu)
          cp "$f" "$staging/usr/share/bash-completion/completions/clipcat-menu.bash"
          ;;
        clipcat-notify)
          cp "$f" "$staging/usr/share/bash-completion/completions/clipcat-notify.bash"
          ;;
        *)
          cp "$f" "$staging/usr/share/bash-completion/completions/"
          ;;
      esac
    done

    cp ${clipcat-static}/share/fish/vendor_completions.d/* "$staging/usr/share/fish/vendor_completions.d/"
    cp ${clipcat-static}/share/zsh/site-functions/* "$staging/usr/share/zsh/site-functions/"

    mkdir -p $out
    cd "$staging"
    nfpm package -f ${nfpmConfig} --packager ${packager} --target "$out"

    runHook postInstall
  '';

  meta = with lib; {
    description = "Clipboard manager written in Rust Programming Language (statically linked, ${packager} package)";
    homepage = "https://github.com/xrelkd/clipcat";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    maintainers = with maintainers; [ xrelkd ];
  };
}
