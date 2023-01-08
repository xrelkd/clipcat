{ mkShell, clipcat, clippy }:

mkShell {
  inputsFrom = [ clipcat ];
  buildInputs = [
    clippy
  ];
  # needed for internal protobuf c wrapper library
  inherit (clipcat)
    PROTOC
    PROTOC_INCLUDE;
}
