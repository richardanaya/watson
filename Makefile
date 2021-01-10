build:
	cargo clippy
	cargo build
test:
	python3 generate_spec_tests.py
	cargo test
