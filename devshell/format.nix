{ pkgs, }:

pkgs.runCommandNoCC "check-format"
{
  buildInputs = with pkgs; [
    fd

    shellcheck

    buf
    nixpkgs-fmt
    nodePackages.prettier
    shfmt
    taplo
    treefmt
  ];
} ''
  treefmt \
    --allow-missing-formatter \
    --fail-on-change \
    --no-cache \
    --formatters prettier \
    --formatters protobuf \
    --formatters nix \
    --formatters shell \
    --formatters hcl \
    --formatters toml \
    -C ${./..}

  # it worked!
  touch $out
''
