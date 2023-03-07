#!/bin/bash
echo "正在生成chain spec..."
./target/release/dorafactory-polkadot build-spec --chain staging --raw --disable-default-bootnode > dora-dot-mainnet.json
echo "正在生成genesis..."
./target/release/dorafactory-polkadot export-genesis-state --chain staging > dora-2087-dot-genesis-live
echo "正在生成wasm..."
./target/release/dorafactory-polkadot export-genesis-wasm --chain staging > dora-2087-dot-wasm-live
echo "完成！"
