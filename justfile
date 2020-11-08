alias b := bench
alias e := echo

build:
    @cargo build
test:
    @cargo test -- --nocapture
lint:
    @cargo clippy
fmt:
    @cargo fmt
echo:
    @RUST_LOG=release cargo run --release --example echo -- serve tcp://127.0.0.1:7878
bench:
    @RUST_LOG=info cargo run --release --example qps -- -c 1000000 -s 1024 tcp://127.0.0.1:7878
