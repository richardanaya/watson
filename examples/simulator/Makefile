build:
	@RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
	@cp target/wasm32-unknown-unknown/release/simulator.wasm .
lint:
	@cargo fmt

serve:
	python -m http.server 8080
