group "default" {
  targets = ["rustatsu-sync"]
}

variable "TAG" {
  default = "1.0.3"
}

variable "REGISTRY" {
  default = "abduzzy"
}

// Special target: https://github.com/docker/metadata-action#bake-definition
target "docker-metadata-action" {}

target "rustatsu-sync" {
  inherits = ["docker-metadata-action"]
  context = "."
  dockerfile = "Dockerfile"
  platforms = [
    "linux/amd64",
    "linux/arm64"
  ]
  tags = [
    "${REGISTRY}/rustatsu-sync:${TAG}",
    "${REGISTRY}/rustatsu-sync:${TAG}-static",
    "${REGISTRY}/rustatsu-sync:latest"
  ]
  labels = [
    "org.opencontainers.image.created" = "${timestamp()}",
    "org.opencontainers.image.version" = TAG
  ]
  attest = [
    "type=provenance,mode=max",
    "type=sbom",
  ]
  output = ["type=registry"]  # registry is alternative of type=image,push=true
}
