{ name
, version
, dockerTools
, fixa
, buildEnv
, ...
}:

dockerTools.buildImage {
  inherit name;
  tag = "v${version}";

  copyToRoot = buildEnv {
    name = "image-root";
    paths = [ fixa ];
    pathsToLink = [ "/bin" ];
  };

  config = {
    Entrypoint = [ "${fixa}/bin/fixa" ];
  };
}
