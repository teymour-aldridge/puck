(cd examples/server && cargo build --target wasm32-wasi)
lunatic target/wasm32-wasi/debug/server.wasm
