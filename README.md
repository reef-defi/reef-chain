## Reef Chain

This repository contains Substrate based runtime for Reef Chain.

### Install

You can install the compiler and the toolchain with:
```bash
make init
```

### Start a development node

The `make run` command will launch a temporary node and its state will be discarded after you terminate the process.
```bash
make run
```
To run the temporary node with ethereum compatibility enabled run:
```bash
make eth
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

### Run in debugger

```bash
make debug
```

### Embedded docs

Once the project has been built, the following command can be used to explore all parameters and subcommands:

```bash
./target/release/reef-node -h
```
