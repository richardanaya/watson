run:
	cargo build --release

wasi:
	cargo build --target=wasm32-wasi --release
	cp target/wasm32-wasi/release/wq.wasm .

test:
	./target/release/wq simplest.wasm > test.json
	./target/release/wq test.json test.wasm
	./target/release/wq wq.wasm > test.json
	./target/release/wq test.json test.wasm