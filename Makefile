.PHONY: init
init:
	./init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --all

.PHONY: debug
debug:
	cargo build && RUST_LOG=debug RUST_BACKTRACE=1 gdb --args target/debug/reef-node --dev --tmp -lruntime=debug

.PHONY: run
run:
	RUST_BACKTRACE=1 cargo run -- --dev --tmp

.PHONY: eth
eth:
	RUST_BACKTRACE=1 cargo run --manifest-path node/Cargo.toml --features with-ethereum-compatibility  -- --dev --tmp

.PHONY: build
build:
	cargo build --manifest-path node/Cargo.toml --features with-ethereum-compatibility --release

.PHONY: watch
watch:
	SKIP_WASM_BUILD=1 cargo watch -c -x build

.PHONY: doc
doc:
	SKIP_WASM_BUILD=1 cargo doc --open

.PHONY: release
release:
	rustup install 1.51.0
	rustup default 1.51.0
	rustup install nightly-2021-03-24
	rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-24
	cargo build --manifest-path node/Cargo.toml --features with-ethereum-compatibility --release
