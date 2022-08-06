#!/bin/bash

cp ../../target/release/dorafactory-polkadot .
cp ../../../polkadot/target/release/polkadot .

bash ./regenerateConfig-rococo-local.sh
