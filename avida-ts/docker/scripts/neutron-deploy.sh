#!/bin/bash

BINARY=${BINARY:-neutrond}
BASE_DIR=./data
CHAINID=${CHAINID:-test-1}
NODE=${NODE:-http://neutron-node:26657}
CHAIN_DIR="$BASE_DIR/$CHAINID"
CONTRACT_FILE=${CONTRACT_FILE:-/tmp/.contract.env}

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

# Wait for block
sleep 6

NEW_CONTRACT_ADDRESS=$($BINARY query wasm list-contract-by-code 19 --node $NODE --output json | jq -r '.contracts[-1]')
# Check if CONTRACT_ADDRESS exists in the file
if grep -q "^CONTRACT_ADDRESS=" "$CONTRACT_FILE"; then
  # If it exists, replace it
  sed -i "s/^CONTRACT_ADDRESS=.*/CONTRACT_ADDRESS=\"$NEW_CONTRACT_ADDRESS\"/" "$CONTRACT_FILE"
else
  # If it does not exist, add it to the end of the file
  echo "CONTRACT_ADDRESS=\"$NEW_CONTRACT_ADDRESS\"" >> "$CONTRACT_FILE"
fi
echo "CONTRACT_ADDRESS has been updated in $CONTRACT_FILE"
