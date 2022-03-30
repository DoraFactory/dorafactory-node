#!/bin/bash
echo "正在生成chain spec1..."
./target/release/dorafactory-node build-spec --chain dev --raw --disable-default-bootnode > dora-rococo-testnet.json
echo "正在生成genesis..."
./target/release/dorafactory-node export-genesis-state --chain dev > dora-2115-genesis-roc-local
echo "正在生成wasm..."
./target/release/dorafactory-node export-genesis-wasm --chain dev > dora-2115-wasm-roc-local
echo "完成！"
