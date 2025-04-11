group "default" {
  targets = ["rustatsu-sync"]
}

variable "TAG" {
  default = "1.0.3"
}

variable "REGISTRY" {
  default = "abduzzy"
}

target "rustatsu-sync" {
  context = "."
  dockerfile = "Dockerfile"
  platforms = [
    "linux/amd64",
    # "linux/arm64"
  ]
  tags = [
    "${REGISTRY}/rustatsu-sync:${TAG}",
    "${REGISTRY}/rustatsu-sync:${TAG}-static",
    "${REGISTRY}/rustatsu-sync:latest"
  ]
  # output = ["type=registry"]  # registry is alternative of type=image,push=true
  provenance = true
  sbom = true
}
