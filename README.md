# dorafactory-node
A private chain based on substrate, with Frame V2

## How to set up
### 1. clone submodule repo
```bash
git clone https://github.com/DoraFactory/dorafactory-node.git
## download submodules
git submodule update --init --recursive
## pull latest submodule repo commit
git submodule update --remote
```
### 2. compile and run
```
> cargo build --release
```

## Start Relaychain

```
./target/release/polkadot --chain rococo-local-cfde.json --alice --tmp --port 30333 --ws-port 9944
./target/release/polkadot --chain rococo-local-cfde.json --bob --tmp --port 30334 --ws-port 9945
./target/release/polkadot --chain rococo-local-cfde.json --dave --tmp --port 30335 --ws-port 9946
```


## Start dorafactory parachain

### generate chain spec
```
./target/release/parachain-collator build-spec --disable-default-bootnode > rococo-local-parachain-plain.json
```

### modify the paraId
```
// --snip--
  "para_id": 2000, // <--- your already registered ID
  // --snip--
      "parachainInfo": {
        "parachainId": 2000 // <--- your already registered ID
      },
  // --snip--
```
### generate a raw chain spec derived from your modified plain chain spec
```
./target/release/parachain-collator build-spec --chain rococo-local-parachain-plain.json --raw --disable-default-bootnode > rococo-local-parachain-2000-raw.json
```

### obtain the wasm
```
./target/release/parachain-collator export-genesis-wasm --chain rococo-local-parachain-2000-raw.json > para-2000-wasm
```

### obtain the genesis state
```
./target/release/parachain-collator export-genesis-state --chain rococo-local-parachain-2000-raw.json > para-2000-genesis
```

### start the collator
```
./target/release/parachain-collator \
--alice \
--collator \
--force-authoring \
--chain rococo-local-parachain-2000-raw.json \
--base-path /tmp/parachain/alice \
--port 40333 \
--ws-port 8844 \
-- \
--execution wasm \
--chain ../../polkadot/rococo-local-cfde.json \
--port 30343 \
--ws-port 9977
```

then the parachain can not produce block, we need register our parachain in the relaychain
https://docs.substrate.io/tutorials/v3/cumulus/connect-parachain/