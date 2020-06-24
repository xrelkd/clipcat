{ pkgs, }:

pkgs.runCommandNoCC "check-format"
{
  buildInputs = with pkgs; [
    fd

    shellcheck

    nixpkgs-fmt
    nodePackages.prettier
    shfmt
    sleek
    taplo
    treefmt
  ];
} ''
  treefmt \
    --allow-missing-formatter \
    --fail-on-change \
    --no-cache \
    --formatters \
      prettier \
      clang-format \
      nix \
      shell \
      hcl \
      toml \
    -C ${./..}

  echo "Checking SQL format with \`sleek\`"
  fd --glob '**/*.{sql}' ${./..} | xargs sleek --check --uppercase --indent-spaces=4 --lines-between-queries=2 --trailing-newline
  echo

  # it worked!
  touch $out
''
