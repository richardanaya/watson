wasi:
	@cargo build --target=wasm32-wasi --release
	@cp target/wasm32-wasi/release/bf.wasm .
test:wasi
	@wasmer run  bf.wasm --dir . -- helloworld.bf helloworld.wasm
serve:test
	@python3 -m http.server 8080
