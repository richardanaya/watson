run:
	cargo run simplest.wasm

wasi:
	cargo build --target=wasm32-wasi --release
	cp target/wasm32-wasi/release/wq.wasm .
