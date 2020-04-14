wasi:
	@cargo build --target=wasm32-wasi --release
	@cp target/wasm32-wasi/release/bfi.wasm .
	@wasmer bfi.wasm --dir . -- helloworld.wasm
	
