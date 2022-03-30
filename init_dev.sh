#!/bin/bash
echo "正在生成chain spec1..."
./target/release/dorafactory-node build-spec --chain ksm --disable-default-bootnode > rococo-local-parachain-2008-plain.json
echo "正在生成chain spec2..."
./target/release/dorafactory-node build-spec --chain rococo-local-parachain-2008-plain.json --raw --disable-default-bootnode > rococo-local-parachain-2008-raw.json
echo "正在生成genesis..."
./target/release/dorafactory-node export-genesis-state --chain rococo-local-parachain-2008-raw.json > para-2008-genesis
echo "正在生成wasm..."
./target/release/dorafactory-node export-genesis-wasm --chain rococo-local-parachain-2008-raw.json > para-2008-wasm
echo "完成！"
