# Testing Commands for proxy-xet

## Code Logic Tests

### Run All Tests
```bash
# Run all 106 unit tests
zig build test --summary all

# Run with optimizations (faster)
zig build test -Doptimize=ReleaseFast
```

### Build All Executables
```bash
# Build everything
zig build --summary all

# Build with release optimization
zig build -Doptimize=ReleaseFast
```

## Rust Proxy Server Tests

### Build the Rust Proxy
```bash
cd proxy-rust
cargo build --release
cd ..
```

### Run the Rust Proxy Server
```bash
# Set environment variables
export HF_TOKEN=your_token_here
export PORT=8080
export ZIG_BIN_PATH=/Users/leoarsenin/proxy-xet/zig-out/bin/xet-download

# Run the server
./proxy-rust/target/release/xet-proxy
```

### Test Proxy Endpoints

#### Health Check
```bash
curl http://localhost:8080/health
# Expected: {"status":"ok","version":"0.1.0"}
```

#### Root Page (HTML)
```bash
curl http://localhost:8080/
# Expected: HTML page with usage instructions
```

#### Download File by Repository Path
```bash
# Example: Download MiMo-7B model
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf \
  -o test-download.gguf

# This will:
# 1. Call xet-download to list files and get the XET hash
# 2. Download the file using the XET hash
# 3. Stream the file back to the client
```

#### Download File by XET Hash
```bash
# If you know the XET hash (64 hex characters)
curl http://localhost:8080/download-hash/89dbfa4888600b29be17ddee8bdbf9c48999c81cb811964eee6b057d8467f927 \
  -o test-download.bin
```

## Quick Test Script

```bash
# Run automated tests
/tmp/test-proxy-simple.sh

# This tests:
# - Server startup
# - Health endpoint
# - Root HTML page
# - Server logs
```

## Known Issues

### TLS Certificate Error
If you see `CertificateSignatureNamedCurveUnsupported`:

**Cause:** Cloudflare WARP or VPN intercepting HTTPS connections

**Solutions:**
1. Disable WARP/VPN temporarily for testing
2. Use the WARP-compatible Docker build (see DOCKER.md)
3. Extract and inject WARP certificates (see scripts/extract-warp-cert.sh)

### Testing Without Network Issues

If you have WARP/VPN that can't be disabled:

```bash
# Use the already downloaded file for testing
ls -lh MiMo-7B-RL-Q8_0.gguf
# Should show: 8106510944 bytes (7.55 GB)

# Or use Docker with WARP support
docker-compose -f docker-compose.warp.yml up
```

## Command Reference

### Zig Executables (in zig-out/bin/)

1. **xet** - Main XET CLI
2. **xet-download** - Download files from HuggingFace
   ```bash
   ./zig-out/bin/xet-download jedisct1/MiMo-7B-RL-GGUF
   ```

3. **download_model** - Single-threaded download
   ```bash
   ./zig-out/bin/download_model jedisct1/MiMo-7B-RL-GGUF MiMo-7B-RL-Q8_0.gguf
   ```

4. **download_model_parallel** - Multi-threaded download
   ```bash
   ./zig-out/bin/download_model_parallel jedisct1/MiMo-7B-RL-GGUF MiMo-7B-RL-Q8_0.gguf
   ```

5. **file_to_xorb** - Convert file to xorb format
   ```bash
   ./zig-out/bin/file_to_xorb input.bin output.xorb
   ```

6. **benchmark** - Performance benchmarks
   ```bash
   ./zig-out/bin/benchmark
   ```

7. **proxy_server** - Zig-based proxy (simple version)
   ```bash
   ./zig-out/bin/proxy_server
   ```

### Rust Proxy Server

**Binary:** `./proxy-rust/target/release/xet-proxy`

**Environment Variables:**
- `HF_TOKEN` (required) - HuggingFace API token
- `PORT` (optional, default: 8080) - Server port
- `ZIG_BIN_PATH` (optional, default: /usr/local/bin/xet-download) - Path to xet-download binary

**Endpoints:**
- `GET /health` - Health check
- `GET /` - Usage instructions (HTML)
- `GET /download/:owner/:repo/*file` - Download by repo and path
- `GET /download-hash/:hash` - Download by XET hash (64 hex chars)

## Testing Checklist

- [x] All Zig unit tests pass (106/106)
- [x] All executables build successfully (15/15)
- [x] Rust proxy builds successfully
- [x] Rust proxy health endpoint works
- [x] Rust proxy root page returns HTML
- [ ] Rust proxy file download (requires WARP disabled or certificate injection)
- [ ] Rust proxy hash-based download (requires WARP disabled or certificate injection)

## Next Steps for Full Testing

1. **Disable WARP/VPN** and test actual file downloads:
   ```bash
   ./proxy-rust/target/release/xet-proxy &
   curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf -o test.gguf
   ```

2. **Or use Docker with WARP support:**
   ```bash
   ./scripts/extract-warp-cert.sh  # One-time setup
   docker-compose -f docker-compose.warp.yml up
   ```

3. **Or test with the already downloaded file:**
   ```bash
   # Verify the existing download
   ls -lh MiMo-7B-RL-Q8_0.gguf
   # Use it for local testing
   ```
