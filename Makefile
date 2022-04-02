.PHONY: configure-rust
configure-rust:
	rustup install 1.59.0
	rustup default 1.59.0
	rustup toolchain install nightly-2022-04-02
	rustup target add wasm32-unknown-unknown --toolchain nightly-2022-04-02
	rustup component add clippy

.PHONY: init
init:
	make configure-rust
	git submodule update --init --recursive

.PHONY: release
release:
	make configure-rust
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
	./target/release/reef-node build-spec --chain testnet-new > assets/chain_spec_testnet.json
	./target/release/reef-node build-spec --chain mainnet-new > assets/chain_spec_mainnet.json
	./target/release/reef-node build-spec --chain testnet-new --raw > assets/chain_spec_testnet_raw.json
	./target/release/reef-node build-spec --chain mainnet-new --raw > assets/chain_spec_mainnet_raw.json

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check

.PHONY: clippy
clippy:
	SKIP_WASM_BUILD=1 cargo clippy -- -D warnings -A clippy::from-over-into -A clippy::unnecessary-cast -A clippy::identity-op -A clippy::upper-case-acronyms

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

.PHONY: fork
fork:
	npm i --prefix fork fork
ifeq (,$(wildcard fork/data))
	mkdir fork/data
endif
	cp target/release/reef-node fork/data/binary
	cp target/release/wbuild/reef-runtime/reef_runtime.compact.wasm fork/data/runtime.wasm
	cp assets/types.json fork/data/schema.json
	cp assets/chain_spec_$(chain)_raw.json fork/data/genesis.json
	cd fork && npm start && cd ..
