OS="`uname -p`"

echo "Building for arch = $OS"

if [ $OS = 'arm' ]; then
  OPTIMIZER="nymlab/optimizer-arm64:0.15.1-clang"
else
  OPTIMIZER="nymlab/optimizer:0.15.1-clang"
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  $OPTIMIZER
