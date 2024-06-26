#!/bin/bash

set -ex

RELAYER_CONFIG="/home/hermes/app/docker/hermes-relayer-config/config.toml"
NEUTRON_RELAYER="/home/hermes/app/docker/hermes-relayer-config/neutron-relayer"
CHEQD_RELAYER="/home/hermes/app/docker/hermes-relayer-config/cheqd-relayer"

CHEQD_CHAIN_ID="cheqd-local-1"
NEUTRON_CHAIN_ID="neutron-local-1"
PATH_NAME="avida-cheqd-neutron"
CHEQD_RESOURCE_PORT="cheqdresource"
AVIDA_SDJWT_PORT_BASE="wasm."
CHANNEL_VERSION="cheqd-resource-v3"


# Function to check if the file is empty or does not contain a non-empty CONTRACT_ADDRESS
check_contract_file() {
  # Check if the file is empty
  if [ ! -s "$CONTRACT_FILE" ]; then
    return 1
  fi

   # Extract CONTRACT_ADDRESS if present and has a non-empty value
  CONTRACT_ADDRESS=$(grep -oE '^CONTRACT_ADDRESS="[^"]*"$' "$CONTRACT_FILE" | cut -d'=' -f2- | tr -d '"')

  if [ -z "$CONTRACT_ADDRESS" ]; then
    return 1
  fi

  return 0
}

# Wait until the file is not empty and contains CONTRACT_ADDRESS with a non-empty value
while ! check_contract_file; do
  echo "Waiting for local.contract to have a non-empty CONTRACT_ADDRESS..."
  sleep 5  # Wait for 5 seconds before checking again
done

AVIDA_SDJWT_PORT="$AVIDA_SDJWT_PORT_BASE$CONTRACT_ADDRESS"

echo "HERMES: Got contract, starting to create channel on path $PATH_NAME with port $AVIDA_SDJWT_PORT..."

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

echo "AVIDA Path created! ($PATH_NAME)"


hermes --config $RELAYER_CONFIG start
