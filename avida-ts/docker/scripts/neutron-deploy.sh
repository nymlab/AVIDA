#!/bin/bash

BINARY=${BINARY:-neutrond}
BASE_DIR=./data
CHAINID=${CHAINID:-test-1}
NODE=${NODE:-http://neutron-node:26657}
CHAIN_DIR="$BASE_DIR/$CHAINID"



/opt/neutron/network/init.sh

# Wait for chain to add all default contracts
sleep 15

$BINARY tx wasm store /tmp/artifacts/avida_sdjwt_verifier.wasm \
  --chain-id $CHAINID \
  --node $NODE \
  --from demowallet2 \
  --gas "auto" \
  --gas-adjustment 1.1  \
  --gas-prices "0.05untrn" \
  --keyring-backend test \
  --home $CHAIN_DIR \
  -y

# Wait for block
sleep 6

$BINARY tx wasm instantiate2 19 '{"max_presentation_len": 30000, "init_registrations": []}' avida-local \
  --label "avida-sdjwt-verifier" \
  --chain-id $CHAINID \
  --node $NODE \
  --from demowallet2 \
  --ascii \
  --gas-adjustment 1.1 \
  --gas 300000 \
  --no-admin \
  --gas-prices "0.05untrn" \
  --keyring-backend test \
  --home $CHAIN_DIR \
  -y
