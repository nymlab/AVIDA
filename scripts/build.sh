OS="`uname -p`"

echo "Building for arch = $OS"

if [ $OS = 'arm' ]; then
  OPTIMIZER="nymlab/optimizer:0.15.1-clang"
else
  return 1
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  $OPTIMIZER
