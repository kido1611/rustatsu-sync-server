group "default" {
  targets = ["rustatsu-sync"]
}

variable "TAG" {
  default = "1.0.7"
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
  attest = [
    "type=provenance,mode=max",
    "type=sbom",
  ]
  output = ["type=docker"]  # registry is alternative of type=image,push=true
}
