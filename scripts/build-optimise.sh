#!/bin/bash

OS="`uname -m`"

echo "Building for arch = $OS"

if [ $OS = 'arm64' ]; then
  OPTIMIZER="ghcr.io/nymlab/optimizer-arm64:0.17.0-clang"
elif [ $OS = 'x86_64' ]; then
  # This will need to be updated
  OPTIMIZER="ghcr.io/nymlab/rust-optimizer:39077c998b881011f3db2cb5c1dbe9904e6be8f1"
else
  return 1
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  $OPTIMIZER
