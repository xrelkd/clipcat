{ pkgs }:

pkgs.runCommand "check-format"
  {
    buildInputs = with pkgs; [
      fd

      shellcheck

      buf
      nixfmt
      prettier
      shfmt
      taplo
      treefmt
    ];
  }
  ''
    cp -r ${./..} /tmp/check-format-src
    chmod -R u+w /tmp/check-format-src

    treefmt \
      --allow-missing-formatter \
      --fail-on-change \
      --no-cache \
      --formatters prettier \
      --formatters nix \
      --formatters shell \
      --formatters hcl \
      --formatters toml \
      -C /tmp/check-format-src

    # it worked!
    touch $out
  ''
