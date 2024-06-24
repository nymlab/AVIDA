#!/bin/bash

set -ex

rm -rf ${CHEQD_HOME}

# initialize wasmd configuration files
cheqd-noded init localnet --default-denom ${CHEQD_DENOM} --chain-id ${CHEQD_CHAIN_ID} --home ${CHEQD_HOME} --overwrite

# add minimum gas prices config to app configuration file
sed -i -r 's/enabled-unsafe-cors = false/enabled-unsafe-cors = true/' ${CHEQD_HOME}/config/app.toml
sed -i -r 's/log_level = "error"/log_level = "debug"/' ${CHEQD_HOME}/config/config.toml



cheqd-noded genesis add-genesis-account $RELAYER_ADDR  2000000000000000000000${CHEQD_DENOM} --home ${CHEQD_HOME}
cheqd-noded genesis add-genesis-account $RESOURCE_ADDR 2000000000000000000000${CHEQD_DENOM} --home ${CHEQD_HOME}

# setup validator
cheqd-noded keys add validator --keyring-backend ${KEYRING_BACKEND} --keyring-dir ${CHEQD_HOME}
cheqd-noded genesis add-genesis-account validator 2000000000000000000000${CHEQD_DENOM} --keyring-backend ${KEYRING_BACKEND} --home ${CHEQD_HOME}
cheqd-noded genesis gentx validator 10000000000000000${CHEQD_DENOM} --chain-id ${CHEQD_CHAIN_ID} --home ${CHEQD_HOME} --keyring-backend ${KEYRING_BACKEND} --amount 10000000000000000${CHEQD_DENOM}

# collect-genesis transactions
cheqd-noded genesis collect-gentxs --home ${CHEQD_HOME}
cheqd-noded genesis validate-genesis --home ${CHEQD_HOME}

cat .cheqdnode/config/genesis.json

cheqd-noded start --home ${CHEQD_HOME} --log_level="trace" --rpc.laddr tcp://0.0.0.0:26657 --grpc.address 0.0.0.0:9090
