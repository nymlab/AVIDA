.PHONY: build-optimise build schemas

# Use the optimizer to build contracts
build-optimise:
	- ./scripts/build-optimise.sh

# Building the contracts with target wasm32-unknown-unknown
build:
	cargo wasm


# Create or update the JSON schemas for the contracts
schemas:
	- ./scripts/schemas.sh
