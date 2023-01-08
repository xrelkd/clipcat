{ mkShell, clipcat }:

mkShell {
  inputsFrom = [ clipcat ];
  # needed for internal protobuf c wrapper library
  inherit (clipcat)
    PROTOC
    PROTOC_INCLUDE;
}
