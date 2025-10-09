#!/usr/bin/env bash
set -euo pipefail

# Usage: ./build_wasm.sh example_name
PKG=${1:-}
if [[ -z "$PKG" ]]; then
  echo "Usage: $0 <example_name>"
  exit 1
fi

# 1. Build the example for wasm
echo "🔧 Building example '$PKG'..."
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' \
cargo build --example "$PKG" --target wasm32-unknown-unknown --profile release-wasm

# 2. Prepare output folder
OUT="build/wasm/$PKG"
mkdir -p "$OUT"

# 3. Run wasm-bindgen
echo "🧩 Running wasm-bindgen..."
wasm-bindgen --target web --out-dir "$OUT" \
  "target/wasm32-unknown-unknown/release-wasm/examples/${PKG}.wasm"

# 4. Generate minimal HTML launcher
HTML_FILE="$OUT/index.html"
cat > "$HTML_FILE" <<HTML
<!doctype html>
<html lang="en">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${PKG^}</title>
<style>
  html,body { height:100%; margin:0; background:#0b0c10; color:#e8eaf0; overflow:hidden }
  #bevy { position:fixed; inset:0; width:100dvw; height:100dvh; display:block }
  .hint { position:fixed; top:10px; left:12px; font:13px/1.4 system-ui,sans-serif; opacity:.6 }
</style>
<canvas id="bevy"></canvas>
<div class="hint">Loading…</div>
<script type="module">
  import init from "./${PKG}.js";
  init().then(() => document.querySelector('.hint')?.remove())
        .catch(e => (console.error(e),
          document.querySelector('.hint').textContent = "Failed to load"));
</script>
</html>
HTML

echo "✅ Build complete!"