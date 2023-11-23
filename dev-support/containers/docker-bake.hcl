group "default" {
  targets = ["clipcat", "clipcat-distroless"]
}

variable "TAG" {
  default = "develop"
}

variable "CONTAINER_REGISTRY" {
  default = "ghcr.io/xrelkd"
}

target "clipcat" {
  dockerfile = "dev-support/containers/alpine/Containerfile"
  platforms  = ["linux/amd64"]
  target     = "clipcat"
  tags       = ["${CONTAINER_REGISTRY}/clipcat:${TAG}"]
  contexts = {
    sccache = "docker-image://ghcr.io/thxnet/ci-containers/sccache:0.5.4"
    rust    = "docker-image://docker.io/library/rust:1.74.0-alpine3.18"
    alpine  = "docker-image://docker.io/library/alpine:3.18"
  }
  args = {
    RUSTC_WRAPPER         = "/usr/bin/sccache"
    AWS_ACCESS_KEY_ID     = null
    AWS_SECRET_ACCESS_KEY = null
    SCCACHE_BUCKET        = null
    SCCACHE_ENDPOINT      = null
    SCCACHE_S3_USE_SSL    = null
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

target "clipcat-distroless" {
  dockerfile = "dev-support/containers/distroless/Containerfile"
  platforms  = ["linux/amd64"]
  target     = "clipcat"
  tags       = ["${CONTAINER_REGISTRY}/clipcat:${TAG}-distroless"]
  contexts = {
    sccache    = "docker-image://ghcr.io/thxnet/ci-containers/sccache:0.5.4"
    rust       = "docker-image://docker.io/library/rust:1.74-slim-buster"
    distroless = "docker-image://gcr.io/distroless/cc-debian11:latest"
  }
  args = {
    RUSTC_WRAPPER         = "/usr/bin/sccache"
    AWS_ACCESS_KEY_ID     = null
    AWS_SECRET_ACCESS_KEY = null
    SCCACHE_BUCKET        = null
    SCCACHE_ENDPOINT      = null
    SCCACHE_S3_USE_SSL    = null
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
