.PHONY: rundbg test clean

rundbg: ##Run debug version
	cargo build
	./target/debug/bradar kenf1/bytes-radar

test: ##Tests
	cargo test

clean: ##Tidy + clean
	cargo fmt && cargo clean