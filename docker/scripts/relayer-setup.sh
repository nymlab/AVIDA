#!/bin/bash

set -ex

RELAYER_HOME=".relayer"
CHEQD_CHAIN_ID="cheqd-local-1"
NEUTRON_CHAIN_ID="neutron-local-1"
CHEQD_RELAYER_MNEMONIC="rebuild sand ocean matrix habit trigger total vendor airport once hybrid napkin refuse drive pottery novel misery steel pony sudden vibrant ready witness nation"
NEUTRON_RELAYER_MNEMONIC="alley afraid soup fall idea toss can goose become valve initial strong forward bright dish figure check leopard decide warfare hub unusual join cart"

# if the $Relayer_HOME exists, skip the initialization, else initialize the relayer
if [ -d "$RELAYER_HOME" ]; then
  echo "Relayer home already exists; skipping init"
else
  rly config init --home $RELAYER_HOME

  # add chain configuration files
  rly chains add -f docker/local-chain-config/cheqd.json $CHEQD_CHAIN_ID --home $RELAYER_HOME
  rly chains add -f docker/local-chain-config/neutron.json $NEUTRON_CHAIN_ID --home $RELAYER_HOME

  # replace min-gas-amount: 0 to 10000 in the config file for both chains
  sed -i -r 's/min-gas-amount: 0/min-gas-amount: 10000/' $RELAYER_HOME/config/config.yaml

  rly paths new $CHEQD_CHAIN_ID $NEUTRON_CHAIN_ID avida-cheqd-neutron --home $RELAYER_HOME
fi

 # we will make sure to add the keys only once by checking if the keys already exist in the relayer home/config/keys directory
 if [ -d "$RELAYER_HOME/keys/$CHEQD_CHAIN_ID" ]; then
   echo "Cheqd relayer key already exists; skipping add"
 else
  rly keys restore $CHEQD_CHAIN_ID cheqd-relayer "$CHEQD_RELAYER_MNEMONIC" --home $RELAYER_HOME
 fi
 if [ -d "$RELAYER_HOME/keys/$NEUTRON_CHAIN_ID/" ]; then
   echo "Neutron relayer key already exists; skipping add"
 else
  rly keys restore $NEUTRON_CHAIN_ID neutron-relayer "$NEUTRON_RELAYER_MNEMONIC" --home $RELAYER_HOME
 fi


 ## create a new client & connection, even if config exists
 rly transact client $CHEQD_CHAIN_ID $NEUTRON_CHAIN_ID avida-cheqd-neutron --override --home $RELAYER_HOME
 rly transact connection avida-cheqd-neutron --override --home $RELAYER_HOME
