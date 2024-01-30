group "default" {
  targets = ["clipcat"]
}

target "clipcat" {
  dockerfile = "dev-support/containers/alpine/Containerfile"
  platforms  = ["linux/amd64"]
  target     = "clipcat"
  contexts = {
    rust   = "docker-image://docker.io/library/rust:1.75.0-alpine3.19"
    alpine = "docker-image://docker.io/library/alpine:3.19"
  }
  args = {
    RUSTC_WRAPPER         = "/usr/bin/sccache"
    SCCACHE_GHA_ENABLED   = "off"
    ACTIONS_CACHE_URL     = null
    ACTIONS_RUNTIME_TOKEN = null
  }
  labels = {
    "description"                     = "Container image for Clipcat"
    "image.type"                      = "final"
    "image.authors"                   = "46590321+xrelkd@users.noreply.github.com"
    "image.vendor"                    = "xrelkd"
    "image.description"               = "Clipcat - clipboard manager written in Rust Programming Language"
    "org.opencontainers.image.source" = "https://github.com/xrelkd/clipcat"
  }
}
