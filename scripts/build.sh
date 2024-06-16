#!/bin/bash

OS="`uname -p`"

echo "Building for arch = $OS"

if [ $OS = 'arm' ]; then
  OPTIMIZER="ghcr.io/nymlab/optimizer-arm64:0.15.1-clang"
elif [ $OS = 'x86_64' ]; then
  OPTIMIZER="ghcr.io/nymlab/rust-optimizer:39077c998b881011f3db2cb5c1dbe9904e6be8f1"
else
  return 1
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  $OPTIMIZER


for file in ./artifacts/*; do
  BASENAME=$(basename "$file")
  NEWNAME=$(echo "$BASENAME" | sed 's/-aarch64//')
  mv "$file" "./artifacts/$NEWNAME"
done
