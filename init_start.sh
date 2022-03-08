#!/bin/zsh
print "正在生成chain spec1..."
./target/release/dorafactory-node build-spec --disable-default-bootnode > rococo-local-parachain-plain.json
print "正在生成chain spec2..."
./target/release/dorafactory-node build-spec --chain rococo-local-parachain-plain.json --raw --disable-default-bootnode > rococo-local-parachain-2000-raw.json
print "正在生成genesis..."
./target/release/dorafactory-node export-genesis-state --chain rococo-local-parachain-2000-raw.json > para-2000-genesis
print "正在生成wasm..."
./target/release/dorafactory-node export-genesis-wasm --chain rococo-local-parachain-2000-raw.json > para-2000-wasm    
print "完成！"

