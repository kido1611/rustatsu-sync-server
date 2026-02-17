group "default" {
  targets = ["rustatsu-sync"]
}

// Special target: https://github.com/docker/metadata-action#bake-definition
target "docker-metadata-action" {}

target "rustatsu-sync" {
  inherits = ["docker-metadata-action"]
  context = "."
  dockerfile = "Containerfile"
  platforms = [
    "linux/amd64",
    "linux/arm64"
  ]
  attest = [
    "type=provenance,mode=max",
    "type=sbom",
  ]
}
