#!/bin/bash
echo "正在生成chain spec..."
./target/release/dorafactory-node build-spec --chain dev --raw --disable-default-bootnode > dora-dev-testnet.json
echo "正在生成genesis..."
./target/release/dorafactory-node export-genesis-state --chain dev > dora-2115-dev-genesis
echo "正在生成wasm..."
./target/release/dorafactory-node export-genesis-wasm --chain dev > dora-2115-dev-wasm
echo "完成！"
