#!/bin/bash
echo "正在生成chain spec1..."
./target/release/dorafactory-node build-spec --chain dev --disable-default-bootnode > rococo-local-parachain-plain.json
echo "正在生成chain spec2..."
./target/release/dorafactory-node build-spec --chain rococo-local-parachain-plain.json --raw --disable-default-bootnode > rococo-local-parachain-2000-raw.json
echo "正在生成genesis..."
./target/release/dorafactory-node export-genesis-state --chain rococo-live-parachain-2078-raw.json > para-2078-genesis
echo "正在生成wasm..."
./target/release/dorafactory-node export-genesis-wasm --chain rococo-live-parachain-2078-raw.json > para-2078-wasm
echo "完成！"

