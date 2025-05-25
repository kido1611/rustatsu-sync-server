#!/bin/sh
# example of docker buildx bake command

docker buildx bake --provenance=true --sbom=true --push --progress=plain --set TAG=1.0.4 --set REGISTRY=abduzzy
