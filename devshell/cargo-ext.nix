{ cargoArgs
, unitTestArgs
, lib
, writeShellScriptBin
,
}:

let
  CARGO_ARGUMENTS = lib.strings.concatMapStrings (x: x + " ") cargoArgs;
  UNIT_TEST_ARGUMENTS = lib.strings.concatMapStrings (x: x + " ") unitTestArgs;
in
{
  cargo-build-all = writeShellScriptBin "cargo-build-all" ''
    if [ $# -gt 0 ] && [ "$1" = "build-all" ]; then
      shift
    fi

    cargo version
    rustc --version
    cargo build ${CARGO_ARGUMENTS} "$@"
  '';

  cargo-clippy-all = writeShellScriptBin "cargo-clippy-all" ''
    if [ $# -gt 0 ] && [ "$1" = "clippy-all" ]; then
      shift
    fi

    cargo clippy --version
    rustc --version
    cargo clippy ${CARGO_ARGUMENTS} "$@"
  '';

  cargo-doc-all = writeShellScriptBin "cargo-doc-all" ''
    if [ $# -gt 0 ] && [ "$1" = "doc-all" ]; then
      shift
    fi

    cargo --version
    rustc --version
    cargo doc --workspace --no-deps --bins --all-features "$@"
  '';

  cargo-test-all = writeShellScriptBin "cargo-test-all" ''
    if [ $# -gt 0 ] && [ "$1" = "test-all" ]; then
      shift
    fi

    cargo --version
    rustc --version
    xvfb-run --auto-servernum cargo test ${UNIT_TEST_ARGUMENTS} --no-fail-fast "$@" -- \
      --nocapture \
      --test
  '';

  cargo-nextest-all = writeShellScriptBin "cargo-nextest-all" ''
    if [ $# -gt 0 ] && [ "$1" = "nextest-all" ]; then
      shift
    fi
    export NEXTEST_RETRIES=5

    cargo --version
    rustc --version
    cargo nextest --version
    xvfb-run --auto-servernum cargo nextest run --workspace --no-fail-fast --no-capture "$@"
  '';
}
