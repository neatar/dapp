#!/usr/bin/env bash

set -o errexit

ROOT_PATH=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../../ && pwd )

echo "Building the docker image"
docker build \
  --network host \
  --tag builder \
  "$ROOT_PATH/.docker/builder"

echo "Building the contract"
docker run \
  --network host \
  --volume "$ROOT_PATH:/code" \
  --env HOST_OWNER="$(id -u):$(id -g)" \
  --workdir /code \
  --rm builder \
  "${@:-src/contract/build.sh}"
