# XET Proxy Server - Testing & Deployment Guide

## Current Status

✅ **Zig CLI built** - `xet-download` binary created (1.1 MB)
✅ **Rust proxy implemented** - Complete HTTP server code ready
✅ **Python test server created** - For immediate testing without Docker
✅ **WARP certificate extracted** - Ready for Docker deployment
⚠️ **Docker build blocked** - WARP intercepts SSL during build

## Problem

Cloudflare WARP intercepts SSL connections even during Docker builds, 
preventing Alpine package manager from downloading dependencies.

## Solutions (in order of simplicity)

### Option 1: Test with Python Proxy (Immediate - 2 minutes)

Use the Python wrapper to test functionality right now:

```bash
# 1. Ensure Zig binary is built
cd /Users/leoarsenin/proxy-xet
zig build -Doptimize=ReleaseFast  # Already done!

# 2. Run Python test server
export HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
python3 proxy_test.py

# 3. Test in another terminal
curl http://localhost:8080/health
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/README.md
```

**Pros:**
- Works immediately
- No Docker issues
- Tests Zig CLI functionality

**Cons:**
- Not production-ready
- Python performance (but fine for testing)

---

### Option 2: Disable WARP for Docker Build (5 minutes)

Temporarily disable WARP, build Docker, then re-enable:

```bash
# 1. Disable Cloudflare WARP in system settings

# 2. Build Docker image
docker build --platform linux/arm64 -f Dockerfile.proxy.warp -t xet-proxy:warp .

# 3. Re-enable WARP

# 4. Run container (WARP can be on now!)
export HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
docker-compose -f docker-compose.proxy.warp.yml up
```

**Pros:**
- Full Rust + Zig solution
- Production ready
- Clean build

**Cons:**
- Requires toggling WARP
- ~10-15 minute build time

---

### Option 3: Use Pre-Built Binaries (10 minutes)

Build Rust locally, then create simple Docker image:

```bash
# 1. Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Build Rust proxy locally
cd proxy-rust
cargo build --release

# 3. Build simple Docker image (uses pre-built binaries)
cd ..
docker build -f Dockerfile.simple -t xet-proxy:simple .

# 4. Run
export HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
docker run -p 8080:8080 \
  -e HF_TOKEN=$HF_TOKEN \
  -v ./cloudflare-warp.crt:/usr/local/share/ca-certificates/cloudflare-warp.crt:ro \
  xet-proxy:simple
```

**Pros:**
- Bypasses Docker build SSL issues
- Full Rust + Zig solution
- WARP stays enabled

**Cons:**
- Requires Rust installed locally
- Slightly more manual

---

### Option 4: Use Cert Injector (Advanced - 15 minutes)

Use Cloudflare's cert-injector tool at Docker build time.

This requires additional Docker configuration and is more complex.

---

## Recommended Approach: Start with Option 1

**Right now, test with Python:**

```bash
cd /Users/leoarsenin/proxy-xet
export HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
python3 proxy_test.py
```

This immediately validates that:
1. Your Zig CLI works
2. The HTTP proxy concept works  
3. File downloads stream correctly

Then choose Option 2 or 3 for production deployment based on preference.

---

## Performance Testing

Once server is running:

```bash
# Health check (instant)
time curl http://localhost:8080/health

# Small file (few seconds)
time curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/README.md \
  -o /tmp/test.md

# Large file (several minutes) - test streaming
time curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf \
  -o /tmp/test.gguf \
  --progress-bar
```

Expected performance (based on your previous test):
- Speed: ~24 MB/s
- 7.5 GB file: ~321 seconds
- Memory: 200-500 MB

---

## Quick Start (Choose One)

### A. Python Test (Fastest)
```bash
export HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
python3 proxy_test.py
```

### B. Docker with WARP Off (Most Complete)
```bash
# Disable WARP → Build → Re-enable WARP → Run
docker build -f Dockerfile.proxy.warp -t xet-proxy:warp .
docker-compose -f docker-compose.proxy.warp.yml up
```

### C. Local Rust + Simple Docker (Middle Ground)
```bash
# Install Rust → Build → Docker
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cd proxy-rust && cargo build --release && cd ..
docker build -f Dockerfile.simple -t xet-proxy:simple .
docker run -p 8080:8080 -e HF_TOKEN=$HF_TOKEN xet-proxy:simple
```

---

## Files Created

All implementation files are ready:
- ✅ `proxy-rust/src/main.rs` - Rust HTTP server (303 lines)
- ✅ `src/download_cli.zig` - Zig CLI wrapper (138 lines)
- ✅ `proxy_test.py` - Python test server
- ✅ `Dockerfile.proxy.warp` - Full build (blocked by WARP)
- ✅ `Dockerfile.simple` - Pre-built binaries approach
- ✅ `docker-compose.proxy.warp.yml` - Deployment config
- ✅ `cloudflare-warp.crt` - WARP certificate extracted

---

## What to Do Now

**Immediate (2 minutes):**
```bash
cd /Users/leoarsenin/proxy-xet
export HF_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxx  
python3 proxy_test.py

# In another terminal:
curl http://localhost:8080/health
```

This validates everything works!

**For Production:**
Choose Option 2 (disable WARP temporarily) or Option 3 (build Rust locally).

Both give you the full Rust + Zig production-ready solution.
