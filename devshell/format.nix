{ pkgs, }:

pkgs.runCommandNoCC "check-format"
{
  buildInputs = with pkgs; [
    fd

    shellcheck

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
    --formatters clang-format \
    --formatters nix \
    --formatters shell \
    --formatters hcl \
    --formatters toml \
    -C ${./..}

  # it worked!
  touch $out
''
