#!/bin/bash
echo "⚙️  Running juno_local on Docker..."

docker run -d \
--name juno_local \
-p 1317:1317 \
-p 26656:26656 \
-p 26657:26657 \
-e STAKE_TOKEN=ujunox \
-e UNSAFE_CORS=true \
-e CHAIN_ID=juno-local \
-e TIMEOUT_COMMIT=100ms \
ghcr.io/cosmoscontracts/juno:15.0 \
./setup_and_run.sh \
juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y juno1tcxyhajlzvdheqyackfzqcmmfcr760malxrvqr   juno1qwwx8hsrhge9ptg4skrmux35zgna47pwnhz5t4 juno1wk2r0jrhuskqmhc0gk6dcpmnz094sc2aq7w9p6 juno1ucl9dulgww2trng0dmunj348vxneufu50c822z juno1yjammmgqu62lz4sxk5seu7ml4fzdu7gkp967q juno1dfd5vtxy2ty5gqqv0cs2z23pfucnpym9kcq8vv juno1ndxfpxzxg267ujpc6wwhw9fs2rvgfh06z6zs25;

