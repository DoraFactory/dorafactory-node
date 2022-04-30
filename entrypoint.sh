#!/bin/bash
PARA_CHAIN="${PARA_CHAIN:-dev}"
RELAY_CHAIN="${RELAY_CHAIN:-/chain-data/rococo-dev.json}"
BASE_PATH="${BASE_PATH:-/chain-data/parachain/alice}"
NODE_NAME="${NODE_NAME:-Alice}"

echo "Starting with config: $PARA_CHAIN, $RELAY_CHAIN, $BASE_PATH, $NODE_NAME"
dorafactory-node \
--name $NODE_NAME \
--collator \
--force-authoring \
--chain $PARA_CHAIN \
--base-path $BASE_PATH --log=info,xcm=trace,xcm-executor=trace \
--port 3033 \
--ws-port 9944 --unsafe-ws-external --rpc-cors all \
-- \
--execution wasm \
--chain $RELAY_CHAIN