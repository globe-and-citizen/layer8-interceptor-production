bench: install_wasi fibonacci_bench

install_wasi:
	which wasmer || cargo install wasmer-cli
	cargo install cargo-wasi

BENCHES := fibonacci formdata

bench_targets := $(addsuffix _bench,$(BENCHES))

$(bench_targets): %_bench:
	@echo "\033[1;34mPorting the $* benchmark to WebAssembly...\033[0m"
	@cargo build --bench=$* --release --target wasm32-wasip1
	@cp ./target/wasm32-wasip1/release/deps/$**.wasm ./benches/$*.wasm || true
	@wasmer run --dir=./benches $*.wasm -- --bench

.PHONY: $(bench_targets)

fibonacci_bench:
	@echo "\033[1;34mRunning the Fibonacci benchmark...\033[0m"
	@cargo wasi run --bench fibonacci --release --target wasm32-wasip1
	@cp ./target/wasm32-wasip1/release/deps/fibonacci.wasm ./benches/fibonacci.wasm || true
	@wasmer run --dir=./ ./benches/fibonacci.wasm -- --bench
