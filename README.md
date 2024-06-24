# AVIDA

This repo contain a contract that is IBC enabled and can query [cheqd] for resources,
including those of AnonCreds and sd-jwt.

[vectis]: https://github.com/nymlab/vectis
[cheqd]: https://cheqd.io


## Usage

## Key difference

- `exp` is not required in the `sd-jwt` token, however, the expiration can be added as a block height / block time. suggest block height in the `RouteVerificationReq` instead
