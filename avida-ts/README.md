# Typescript library for AVIDA


## Local development

Please make sure you have built the contracts before running the local demo.

```bash
# at the root of the repo
make build
```

You also have to set write permission on the contract addresss file:
```bash
echo CONTRACT_ADDRESS= > avida-ts/env/local.contract
```

### Create local networks

This step creates a local cheqd node, a local standalone neutron node and a relayer.
All chain config files are in `docker/local-chain-config`.

Then we can bring up docker containers:

```bash
# starting in ./avida-ts
docker-compose -f ./docker/docker-compose.local.yaml up -d
```
This does several things
1. `scripts/cheqd-genesis.sh` creates a cheqd node with a genesis tx to pre-fund the relayer and resource owner accounts.
2. `scripts/cheqd-resource-uploader.sh` uploads the demo cheqd resource from `docker/cheqd-resource-artifacts`.
3. `scripts/neutron-deploy.sh` deploys the `avida-sdjwt-verifier` contract to neutron and sets the contract address in `./local.contract`
4. `scripts/relayer-avida-link.sh` waits for the new contract address in `./contract.env` and when available create a client & connection & channel, then starts relaying messages between the cheqd/x/resource module and the `avida-sdjwt-verifier` contract.
5. a neutron node also gets started without custom scripts as see in the `docker-compose.local.yaml` file


> The local network takes a few mins to start up, please wait for the relayer to start before running the demo.

## Run demo

`avida-ts` directory has packages to support the demo.

> The local network takes a few mins to start up, please wait for the relayer to start before running the demo.

```
pnpm i
pnpm run build:packages
pnpm run local-demo
```
