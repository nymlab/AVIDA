vectis-version := "v0.2.1"
cw-plus-version := "vectis-beta-v1" 
vectis-contracts := "vectis_proxy vectis_factory vectis_plugin_registry"
cwplus-contracts := "cw3_fixed_multisig cw3_flex_multisig cw4_group"
set dotenv-load

build: 
  @echo "Building Optimised WASM file(s)"
  if [[ $(uname -m) =~ "arm64" ]]; then \
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    --platform linux/arm64 \
    cosmwasm/workspace-optimizer-arm64:0.13.0; else \
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    --platform linux/amd64 \
    cosmwasm/workspace-optimizer:0.13.0; fi

test contract="":
  cargo test --locked --{{ if contract == "" { "workspace" } else { "package" } }} {{contract}} 

dl-vectis-contracts:
  #!/usr/bin/env bash
  if [ ! -d "vectis-wasm" ]; then \
    mkdir vectis-wasm; \
  fi
  for contract in {{vectis-contracts}}; do \
    URL="https://github.com/nymlab/vectis/releases/download/{{vectis-version}}/$contract.wasm"; \
    echo $URL; \
    curl -Lv $URL -o "./vectis-wasm/$contract.wasm"
  done

dl-cw-plus-contracts:
  #!/usr/bin/env bash
  if [ ! -d "vectis-wasm" ]; then \
    mkdir vectis-wasm; \
  fi
  for contract in {{cwplus-contracts}}; do \
    URL="https://github.com/nymlab/cw-plus/releases/download/{{cw-plus-version}}/$contract.wasm"; \
    echo $URL; \
    curl -Lv $URL -o "./vectis-wasm/$contract.wasm"
  done

#	npm install -g vectisCLI; \
setup: 
  if which node > /dev/null; then \
    just dl-vectis-contracts; \
    just dl-cw-plus-contracts; \
  else \
    echo "node needs to be installed to use vectisCLI"; \
  fi

generate-schemas: 
  for dir in $PWD/contracts/*/*/; do \
    cd $dir \
    cargo run --example schema \
    cd - \
  done

run-local:
  ./scripts/rm-local-node.sh
  ./scripts/run-test-node.sh

stop-local:
  ./scripts/rm-local-node.sh

deploy-vectis network:
  #!/usr/bin/env bash
   vectisCLI {{network}} deploy-vectis 
  




