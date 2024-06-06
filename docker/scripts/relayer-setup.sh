#!/bin/bash

set -ex

RELAYER_HOME=".relayer"
CHEQD_CHAIN_ID="cheqd-local-1"
NEUTRON_CHAIN_ID="neutron-local-1"
CHEQD_RELAYER_MNEMONIC="rebuild sand ocean matrix habit trigger total vendor airport once hybrid napkin refuse drive pottery novel misery steel pony sudden vibrant ready witness nation"
NEUTRON_RELAYER_MNEMONIC="alley afraid soup fall idea toss can goose become valve initial strong forward bright dish figure check leopard decide warfare hub unusual join cart"

# rly config init --home $RELAYER_HOME
#
# rly chains add -f docker/local-chain-config/cheqd.json $CHEQD_CHAIN_ID --home $RELAYER_HOME
# rly chains add -f docker/local-chain-config/neutron.json $NEUTRON_CHAIN_ID --home $RELAYER_HOME
#
# rly keys restore $CHEQD_CHAIN_ID cheqd-relayer "$CHEQD_RELAYER_MNEMONIC" --home $RELAYER_HOME
# rly keys restore $NEUTRON_CHAIN_ID neutron-relayer "$NEUTRON_RELAYER_MNEMONIC" --home $RELAYER_HOME

rly q balance $CHEQD_CHAIN_ID --home $RELAYER_HOME
rly q balance $NEUTRON_CHAIN_ID --home $RELAYER_HOME
