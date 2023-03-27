# Build the rust code into a wasm binary
Write-Output "cargo build..."
cargo build --target wasm32-unknown-unknown

# Use wasm-bindgen to generate javascript glue code for rust and JS code to call each other.
# This creates a javascript file and also a modified version of the wasm
# (removing some metadata that was added during the rust build)
Write-Output "wasm-bindgen..."
wasm-bindgen --target no-modules --out-dir target\wasm32-unknown-unknown\debug\wasm-bindgen target\wasm32-unknown-unknown\debug\tournament-viewer.wasm

# Turn the modifed wasm binary into a javascript hardcoded array of bytes. This means we can load it
# using a file:// origin, without having to set up a web server (we can't load the binary file due to CORS)
Write-Output "JS hardcoded array..."
$wasmBytes = [System.IO.File]::ReadAllBytes("target\wasm32-unknown-unknown\debug\wasm-bindgen\tournament-viewer_bg.wasm")
$js = [System.Text.StringBuilder]::new();
[void]$js.Append("var WASM_BYTES = new Uint8Array([");
foreach ($b in $wasmBytes) {
    [void]$js.Append($b);
    [void]$js.Append(",");
}
[void]$js.AppendLine("]);");
[System.IO.File]::WriteAllText("target\wasm32-unknown-unknown\debug\wasm-bindgen\tournament-viewer_bg.wasm.js", $js);