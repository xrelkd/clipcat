{
  name,
  version,
  dockerTools,
  clipcat,
  buildEnv,
  ...
}:

dockerTools.buildImage {
  inherit name;
  tag = "v${version}";

  copyToRoot = buildEnv {
    name = "image-root";
    paths = [ clipcat ];
    pathsToLink = [ "/bin" ];
  };

  config = {
    Entrypoint = [ "${clipcat}/bin/clipcatd" ];
  };
}
