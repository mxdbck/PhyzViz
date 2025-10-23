MTL HUD for frametimes :
```MTL_HUD_ENABLED=1 cargo run --example ribbon-double-pendulum```


Compile for wasm :
```RUSTFLAGS='--cfg getrandom_backend="wasm_js" -C target-feature=+simd128' \
        cargo run -r --target wasm32-unknown-unknown --example pendulum```

Display wasm :
`wasm-server-runner ./target/wasm32-unknown-unknown/release/examples/pendulum.wasm`

`wasm-opt -O --strip-debug --strip-dwarf --strip-producers -o out_opt.wasm in.wasm`


```
PKG=double-pendulum

RUSTFLAGS='--cfg getrandom_backend="wasm_js" -C target-feature=+simd128' cargo build --example "$PKG" --target wasm32-unknown-unknown --profile release-wasm

OUT=build/wasm/$PKG

mkdir -p "$OUT"

wasm-bindgen --target web --out-dir "$OUT" target/wasm32-unknown-unknown/release-wasm/examples/${PKG}.wasm

wasm-opt "$OUT/${PKG}_bg.wasm" \
  -O \
  --enable-simd \
  --enable-bulk-memory \
  --enable-nontrapping-float-to-int \
  --enable-sign-ext \
  --enable-multivalue \
  --enable-reference-types \
  --strip-debug --strip-dwarf --strip-producers \
  -o "$OUT/${PKG}_bg.wasm.tmp" \
&& mv "$OUT/${PKG}_bg.wasm.tmp" "$OUT/${PKG}_bg.wasm"
```