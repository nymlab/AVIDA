#!/bin/bash

set -ex

RELAYER_HOME="/tmp/.relayer"
CHEQD_CHAIN_ID="cheqd-local-1"
NEUTRON_CHAIN_ID="neutron-local-1"
PATH_NAME="avida-cheqd-neutron"
CHEQD_RESOURCE_PORT="cheqdresource"
AVIDA_SDJWT_PORT_BASE="wasm."
CHANNEL_VERSION="cheqd-resource-v3"

# This is used in avida local deploy
CONTRACT_ADDRESS=${CONTRACT_ADDRESS:-neutron1js0jdjmwtzpp32sfz6x44t2mxv5vzk5mku4ckqge46us9erq097qa60waw}

AVIDA_SDJWT_PORT="$AVIDA_SDJWT_PORT_BASE$CONTRACT_ADDRESS"

# let the nodes start, deploy avida-sdjwt-verifier contract
sleep 30

## create a new client & connection, even if config exists
rly transact client \
  $CHEQD_CHAIN_ID \
  $NEUTRON_CHAIN_ID \
  $PATH_NAME \
  --override \
  --home $RELAYER_HOME

rly transact connection \
$PATH_NAME \
  --override \
  --home $RELAYER_HOME

rly transact channel \
  $PATH_NAME \
  --src-port $CHEQD_RESOURCE_PORT \
  --dst-port $AVIDA_SDJWT_PORT \
  --order unordered \
  --version $CHANNEL_VERSION \
  --override \
  --home $RELAYER_HOME

# sleep 30

# rly start \
#   $PATH_NAME \
#   --home $RELAYER_HOME
