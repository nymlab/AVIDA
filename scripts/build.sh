#!/bin/bash

OS="`uname -p`"

echo "Building for arch = $OS"

if [ $OS = 'arm' ]; then
  OPTIMIZER="cosmwasm/optimizer-arm64:0.16.1"
elif [ $OS = 'x86_64' ]; then
  OPTIMIZER="cosmwasm/optimizer:0.16.1"
else
  return 1
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  $OPTIMIZER