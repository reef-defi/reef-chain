## Reef Chain
Reef chain is written in [Rust](https://www.rust-lang.org/). A basic familiarity with Rust tooling is required.

To learn more about Reef chain, please refer to **[Documentation](https://docs.reef.finance/)**.

### Clone
To clone the repo with its submodules run:
```bash
git clone --recursive https://github.com/reef-defi/reef-chain
```

### Install

You can install the latest compiler and the toolchain with:
```bash
make init
```

### Start a development node

The `make run` command will launch a temporary node and its state will be discarded after you terminate the process.
```bash
make run
```

### Run a persistent single-node chain

Use the following command to build the node without launching it:

```bash
make build
```

This command will start the single-node development chain with persistent state:

```bash
./target/release/reef-node --dev
```

Purge the development chain's state:

```bash
./target/release/reef-node purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/reef-node -lruntime=debug --dev
```

### Run tests

```bash
make test
```

### Run benchmarks

Run runtime benchmark tests:
```bash
make bench
```

Run module benchmark tests:
```bash
cargo test -p module-poc --all-features
```

Run the module benchmarks and generate the weights file:
```
./target/release/reef-node benchmark \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
    --pallet=module_poc \
    --extrinsic='*'  \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output=./modules/poc/src/weights.rs
```

### Run in debugger

```bash
make debug
```

### Embedded docs

Once the project has been built, the following command can be used to explore all parameters and subcommands:

```bash
./target/release/reef-node -h
```

### Release builds

To list all available release builds run:
```bash
git tag
```

To create a corresponding production build, first checkout the tag:
```bash
git checkout testnet-1
```

Then run this command to install appropriate compiler version and produce a binary.
```bash
make release
```
