#!/bin/bash

set -e

echo "👀 Checking and setting up requirements on your machine..."

command -v docker >/dev/null 2>&1 || { echo >&2 "Docker is not installed on your machine, local Juno node can't be ran. Install it from here: https://www.docker.com/get-started"; exit 1; }

NODE_1=`docker ps -a --format="{{.Names}}" | grep juno_local | awk '{print $1}'`

if [[ "$NODE_1" != "" ]]; then
echo "Removing existing node container $NODE_1"
docker rm -f $NODE_1 > /dev/null;
fi

