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

    cp ${clipcat-static}/bin/clipcatd       "$staging/usr/bin/"
    cp ${clipcat-static}/bin/clipcatctl     "$staging/usr/bin/"
    cp ${clipcat-static}/bin/clipcat-menu   "$staging/usr/bin/"
    cp ${clipcat-static}/bin/clipcat-notify "$staging/usr/bin/"

    cp ${clipcat-static}/share/bash-completion/completions/* "$staging/usr/share/bash-completion/completions/"
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
