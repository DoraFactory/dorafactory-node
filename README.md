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
> ./target/release/dorafactory-node --dev --tmp
```