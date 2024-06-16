#!/bin/bash

set -ex

RESOURCE_ADDR="cheqd16kxf0tkkjc6llu072qwkvj5plm7nr0xdhp353q"
CHEQD_HOME="./.cheqdnode"
CHEQD_CHAIN_ID="cheqd-local-1"
KEYRING_BACKEND="test"
RESOURCE_MNEMONIC="differ coconut rate prosper cabbage depth rich gather myself winner evidence buzz alcohol garment wing soup reform glare attitude parrot sunset peasant affair envelope"
NODE=${NODE:-http://cheqd-node:26657}

# add resource key
echo "$RESOURCE_MNEMONIC" | cheqd-noded keys add resource --home "$CHEQD_HOME" --recover --keyring-backend=test

# waiting for first block
sleep 15

cheqd-noded tx \
  cheqd create-did \
  /tmp/cheqd-resource-artifacts/did_payload.json \
  --version-id "60683a87-38c3-4f08-86ad-5aec8f291fa6" \
  --chain-id $CHEQD_CHAIN_ID \
  --keyring-backend $KEYRING_BACKEND \
  --home $CHEQD_HOME \
  --from $RESOURCE_ADDR \
  --gas 200000 \
  --fees 50000000000ncheq \
  --gas-adjustment 1.5 \
  --node $NODE \
  --yes

# waiting for block
sleep 10

cheqd-noded tx \
  resource create \
  /tmp/cheqd-resource-artifacts/resource_payload_no_data.json \
  /tmp/cheqd-resource-artifacts/jwk.json \
  --chain-id $CHEQD_CHAIN_ID \
  --keyring-backend $KEYRING_BACKEND \
  --home $CHEQD_HOME \
  --from $RESOURCE_ADDR \
  --gas 200000 \
  --fees 50000000000ncheq \
  --gas-adjustment 1.5 \
  --node $NODE \
  --yes
