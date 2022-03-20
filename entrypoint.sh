 #!/bin/bash
 ./target/release/dorafactory-node \
--alice \
--collator \
--force-authoring \
--chain rococo-local-parachain-2052-raw.json \
--base-path /tmp/parachain/alice \
--port 40333 \
--ws-port 8844 \
-- \
--execution wasm \
--chain rococo-local-cfde.json \
--port 30343 \
--ws-port 9977