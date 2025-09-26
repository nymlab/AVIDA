#!/bin/bash

OS="`uname -m`"

echo "Building for arch = $OS"

if [ $OS = 'arm64' ]; then
  OPTIMIZER="ghcr.io/nymlab/optimizer-arm64:0.17.0-clang"
elif [ $OS = 'x86_64' ]; then
  OPTIMIZER="cosmwasm/optimizer:0.17.0"
else
  return 1
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  $OPTIMIZER


# for file in ./artifacts/*; do
#   BASENAME=$(basename "$file")
#   NEWNAME=$(echo "$BASENAME" | sed 's/-aarch64//')
#   mv "$file" "./artifacts/$NEWNAME"
# done
