#!/bin/bash

cp ../target/release/dorafactory-node .
cp ../../polkadot/target/release/polkadot .

bash ./regenerateConfig-rococo-local.sh