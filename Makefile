.PHONY: init
init:
	rustup update stable
	rustup update nightly
	rustup target add wasm32-unknown-unknown --toolchain nightly
	git submodule update --init --recursive

.PHONY: release
release:
	rustup install 1.51.0
	rustup default 1.51.0
	rustup toolchain install nightly-2021-05-09
	rustup target add wasm32-unknown-unknown --toolchain nightly-2021-05-09
	rm -rf target/
	cargo build --manifest-path node/Cargo.toml --features with-ethereum-compatibility --release

.PHONY: build
build:
	cargo build --manifest-path node/Cargo.toml --features runtime-benchmarks,with-ethereum-compatibility --release

.PHONY: wasm
wasm:
	cargo build -p reef-runtime --features with-ethereum-compatibility --release

.PHONY: genesis
genesis:
	make release
	./target/release/reef-node build-spec --chain testnet > assets/chain_spec_testnet.json
	./target/release/reef-node build-spec --chain mainnet > assets/chain_spec_mainnet.json

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check

.PHONY: clippy
clippy:
	SKIP_WASM_BUILD=1 cargo clippy

.PHONY: watch
watch:
	SKIP_WASM_BUILD=1 cargo watch -c -x build

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --all

.PHONY: debug
debug:
	cargo build && RUST_LOG=debug RUST_BACKTRACE=1 rust-gdb --args target/debug/reef-node --dev --tmp -lruntime=debug

.PHONY: run
run:
	RUST_BACKTRACE=1 cargo run --manifest-path node/Cargo.toml --features with-ethereum-compatibility  -- --dev --tmp

.PHONY: log
log:
	RUST_BACKTRACE=1 RUST_LOG=debug cargo run --manifest-path node/Cargo.toml --features with-ethereum-compatibility  -- --dev --tmp

.PHONY: noeth
noeth:
	RUST_BACKTRACE=1 cargo run -- --dev --tmp

.PHONY: bench
bench:
	SKIP_WASM_BUILD=1 cargo test --manifest-path node/Cargo.toml --features runtime-benchmarks,with-ethereum-compatibility benchmarking

.PHONY: doc
doc:
	SKIP_WASM_BUILD=1 cargo doc --open

.PHONY: cargo-update
cargo-update:
	cargo update
	cargo update --manifest-path node/Cargo.toml
	make test

