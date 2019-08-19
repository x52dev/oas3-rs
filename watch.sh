RUST_BACKTRACE=1 \
RUST_LOG=warn,oas=trace,conformance=trace \
cargo watch \
-x "$@"
