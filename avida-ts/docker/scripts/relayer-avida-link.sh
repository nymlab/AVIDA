#!/bin/bash

set -ex

RELAYER_CONFIG="/home/hermes/hermes-relayer-config/config.toml"
NEUTRON_RELAYER="/home/hermes/hermes-relayer-config/neutron-relayer"
CHEQD_RELAYER="/home/hermes/hermes-relayer-config/cheqd-relayer"
CHEQD_CHAIN_ID="cheqd-local-1"
NEUTRON_CHAIN_ID="neutron-local-1"
PATH_NAME="avida-cheqd-neutron"
CHEQD_RESOURCE_PORT="cheqdresource"
AVIDA_SDJWT_PORT_BASE="wasm."
CHANNEL_VERSION="cheqd-resource-v3"

# This is used in avida local deploy
CONTRACT_ADDRESS=${CONTRACT_ADDRESS:-neutron1ev8e7z53nm9ncn8jv5efsuv7k5stn7uyt627qmpk9a98netlua0qumsp9n}

AVIDA_SDJWT_PORT="$AVIDA_SDJWT_PORT_BASE$CONTRACT_ADDRESS"


cat $RELAYER_CONFIG

hermes --config $RELAYER_CONFIG \
  keys add \
  --chain $NEUTRON_CHAIN_ID \
  --mnemonic-file $NEUTRON_RELAYER

hermes --config $RELAYER_CONFIG \
  keys add \
  --chain $CHEQD_CHAIN_ID \
  --mnemonic-file $CHEQD_RELAYER

# let the nodes start, deploy avida-sdjwt-verifier contract
sleep 20

hermes --config $RELAYER_CONFIG \
  create channel \
  --a-chain $CHEQD_CHAIN_ID \
  --b-chain $NEUTRON_CHAIN_ID \
  --a-port $CHEQD_RESOURCE_PORT \
  --b-port $AVIDA_SDJWT_PORT \
  --new-client-connection \
  --order ORDER_UNORDERED \
  --channel-version $CHANNEL_VERSION \
  --yes

hermes --config $RELAYER_CONFIG start
