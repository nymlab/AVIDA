# Typescript library for AVIDA


## Local development

## Create local networks

This step creates a local cheqd node, a local standalone neutron node and a relayer.
All chain config files are in `docker/local-chain-config`.

Then we can bring up docker containers:

```bash
docker-compose -f docker/docker-compose.local.yaml up -d
```

## Deploy contracts
