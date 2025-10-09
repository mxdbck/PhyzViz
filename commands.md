Compile for wasm :
```RUSTFLAGS='--cfg getrandom_backend="wasm_js"' \
        cargo run -r --target wasm32-unknown-unknown --example pendulum```

Display wasm :
`wasm-server-runner ./target/wasm32-unknown-unknown/release/examples/pendulum.wasm`


```
PKG=pendulum
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --example pendulum --target wasm32-unknown-unknown --profile release-wasm
OUT=build/wasm/$PKG
mkdir -p "$OUT"
wasm-bindgen --target web --out-dir "$OUT" target/wasm32-unknown-unknown/release-wasm/examples/${PKG}.wasm
```