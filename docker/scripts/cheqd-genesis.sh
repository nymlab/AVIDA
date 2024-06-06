#!/bin/bash

set -ex

# it can be a sepearte script to add to genesis file
# we need to add some keys to the genesis file
# use that file to start the chaink

CHEQD_HOME=".cheqdnode"
CHEQD_CHAIN_ID="cheqd-local-1"
CHEQD_DENOM="ncheq"
KEYRING_BACKEND="test"
RELAYER_MNEMONIC="rebuild sand ocean matrix habit trigger total vendor airport once hybrid napkin refuse drive pottery novel misery steel pony sudden vibrant ready witness nation"
RESOURCE_MNEMONIC="differ coconut rate prosper cabbage depth rich gather myself winner evidence buzz alcohol garment wing soup reform glare attitude parrot sunset peasant affair envelope"

RELAYER_ADDR="cheqd1kd0yrf4p4pm3lfukkx8fj04we87822zkjy2zwe"
RESOURCE_ADDR="cheqd16kxf0tkkjc6llu072qwkvj5plm7nr0xdhp353q"

rm -rf ${CHEQD_HOME}

# initialize wasmd configuration files
cheqd-noded init localnet --default-denom ${CHEQD_DENOM} --chain-id ${CHEQD_CHAIN_ID} --home ${CHEQD_HOME} --overwrite

# add minimum gas prices config to app configuration file
sed -i -r 's/minimum-gas-prices = ""/minimum-gas-prices = "0.01ncheq"/' ${CHEQD_HOME}/config/app.toml
sed -i -r 's/enabled-unsafe-cors = false/enabled-unsafe-cors = true/' ${CHEQD_HOME}/config/app.toml
sed -i -r 's/log_level = "error"/log_level = "debug"/' ${CHEQD_HOME}/config/config.toml



cheqd-noded genesis add-genesis-account $RELAYER_ADDR 10000000000000${CHEQD_DENOM} --home ${CHEQD_HOME}
cheqd-noded genesis add-genesis-account $RESOURCE_ADDR 10000000000000${CHEQD_DENOM} --home ${CHEQD_HOME}

# setup validator
cheqd-noded keys add validator --keyring-backend ${KEYRING_BACKEND} --keyring-dir ${CHEQD_HOME}
cheqd-noded genesis add-genesis-account validator 10000000000000${CHEQD_DENOM} --keyring-backend ${KEYRING_BACKEND} --home ${CHEQD_HOME}
cheqd-noded genesis gentx validator 10000000000000${CHEQD_DENOM} --chain-id ${CHEQD_CHAIN_ID} --home ${CHEQD_HOME} --keyring-backend ${KEYRING_BACKEND} --amount 10000000000000${CHEQD_DENOM}

# collect-genesis transactions
cheqd-noded genesis collect-gentxs --home ${CHEQD_HOME}
cheqd-noded genesis validate-genesis --home ${CHEQD_HOME}

cat .cheqdnode/config/genesis.json

cheqd-noded start --home ${CHEQD_HOME} --log_level="trace" --rpc.laddr tcp://0.0.0.0:26657
