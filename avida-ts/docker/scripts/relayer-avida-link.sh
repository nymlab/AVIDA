#!/bin/bash

set -ex

RELAYER_HOME="/tmp/.relayer"
CHEQD_CHAIN_ID="cheqd-local-1"
NEUTRON_CHAIN_ID="neutron-local-1"

# let the nodes start
sleep 15

 ## create a new client & connection, even if config exists
 rly transact client $CHEQD_CHAIN_ID $NEUTRON_CHAIN_ID avida-cheqd-neutron --override --home $RELAYER_HOME
 rly transact connection avida-cheqd-neutron --override --home $RELAYER_HOME
