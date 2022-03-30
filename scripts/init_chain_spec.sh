#!/bin/bash
echo "正在生成chain spec..."
./target/release/dorafactory-node build-spec --chain staging --raw --disable-default-bootnode > dora-ksm-mainnet.json
echo "正在生成genesis..."
./target/release/dorafactory-node export-genesis-state --chain staging > dora-2115-ksm-genesis-live
echo "正在生成wasm..."
./target/release/dorafactory-node export-genesis-wasm --chain staging > dora-2115-ksm-wasm-live
echo "完成！"
