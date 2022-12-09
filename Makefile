
build: build-wasm
	cargo build -p many-wasm-server

build-wasm: target/wasm32-wasi/debug/
	cargo build --target wasm32-wasi --workspace --exclude many-wasm-server

run: build
	cargo run --bin many-wasm-server -- -v --pem ${HOME}/Sources/temp/id1.pem --bind 127.0.0.1:8000 --create demo/config.json5
