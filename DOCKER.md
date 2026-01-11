# Docker Deployment Guide

This guide explains how to run the XET Proxy Server in Docker with or without Cloudflare WARP.

## Overview

The XET Proxy Server provides an HTTP API to download files from HuggingFace using the XET protocol. It streams files as they are reconstructed, avoiding buffering entire files in memory.

**Two Docker configurations are available:**
1. **Standard** (`Dockerfile`) - Requires Cloudflare WARP to be **disabled**
2. **WARP-Compatible** (`Dockerfile.warp`) - Works with Cloudflare WARP **enabled**

## Quick Start

### Option 1: Without WARP (Standard)

```bash
# Set your HuggingFace token
export HF_TOKEN=hf_xxx

# Disable Cloudflare WARP in system settings

# Build and run
docker-compose up -d

# Test
curl http://localhost:8080/health
```

### Option 2: With WARP (WARP-Compatible)

```bash
# Extract WARP certificate (one-time setup)
./scripts/extract-warp-cert.sh

# Set your HuggingFace token
export HF_TOKEN=hf_xxx

# Keep Cloudflare WARP enabled

# Build and run
docker-compose -f docker-compose.warp.yml up -d

# Test
curl http://localhost:8080/health
```

## API Endpoints

### `GET /health`
Health check endpoint

```bash
curl http://localhost:8080/health
# {"status":"ok","version":"0.1.0"}
```

### `GET /download/:repo_id/:file_path`
Download file by repository and path

```bash
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf \
  -o model.gguf
```

### `GET /download-hash/:xet_hash_hex`
Download file directly by XET hash (64 hex characters)

```bash
curl http://localhost:8080/download-hash/ef62b7509a2c65746d7ccbfaeb75da2385ac669a431532e1da9bcd500f49e5bd \
  -o model.safetensors
```

### `GET /`
Usage instructions (HTML)

```bash
open http://localhost:8080/
```

## Detailed Setup

### WARP Certificate Extraction

The WARP-compatible image requires the Cloudflare WARP certificate to be extracted from your macOS keychain:

```bash
# Extract certificate
./scripts/extract-warp-cert.sh

# Verify extraction
ls -lh cloudflare-warp.crt
openssl x509 -in cloudflare-warp.crt -noout -subject
```

This creates a `cloudflare-warp.crt` file that will be mounted into the Docker container.

### Building Images Manually

```bash
# Standard image
docker build -t xet-proxy:latest -f Dockerfile .

# WARP-compatible image
docker build -t xet-proxy:warp -f Dockerfile.warp .
```

### Running Containers Manually

**Standard (no WARP):**
```bash
docker run -d \
  --name xet-proxy \
  -p 8080:8080 \
  -e HF_TOKEN=hf_xxx \
  xet-proxy:latest
```

**WARP-compatible:**
```bash
docker run -d \
  --name xet-proxy-warp \
  -p 8080:8080 \
  -e HF_TOKEN=hf_xxx \
  -v ./cloudflare-warp.crt:/usr/local/share/ca-certificates/cloudflare-warp.crt:ro \
  xet-proxy:warp
```

## Configuration

### Environment Variables

- `HF_TOKEN` (required): Your HuggingFace API token
- `PORT` (optional): HTTP port to listen on (default: 8080)

### Port Mapping

The default port is `8080`. You can change the host port:

```bash
# Run on port 3000
docker run -p 3000:8080 -e HF_TOKEN=hf_xxx xet-proxy:latest

# Access at http://localhost:3000
```

## Usage Examples

### Download a Model

```bash
# Download MiMo-7B model (7.55 GB)
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf \
  -o MiMo-7B-RL-Q8_0.gguf

# Shows progress with curl
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf \
  -o model.gguf \
  --progress-bar
```

### Download by Hash

```bash
# If you know the XET hash, download directly
curl http://localhost:8080/download-hash/89dbfa4888600b29be17ddee8bdbf9c48999c81cb811964eee6b057d8467f927 \
  -o model.safetensors
```

### Using with wget

```bash
wget http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf \
  -O model.gguf
```

### Python Example

```python
import requests

url = "http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf"

with requests.get(url, stream=True) as r:
    r.raise_for_status()
    with open('model.gguf', 'wb') as f:
        for chunk in r.iter_content(chunk_size=8192):
            f.write(chunk)
```

## Troubleshooting

### Health Check Fails

```bash
# Check logs
docker logs xet-proxy

# Check if container is running
docker ps | grep xet-proxy

# Test manually
curl -v http://localhost:8080/health
```

### Certificate Errors (WARP mode)

```bash
# Verify certificate is mounted
docker exec xet-proxy-warp ls -l /usr/local/share/ca-certificates/

# Check if certificate was added to trust store
docker exec xet-proxy-warp cat /etc/ssl/certs/ca-certificates.crt | grep Cloudflare
```

### HF_TOKEN Not Set

```bash
# Check environment variable is passed
docker exec xet-proxy-warp env | grep HF_TOKEN

# If missing, recreate container with -e HF_TOKEN=hf_xxx
```

### Connection Refused

```bash
# Make sure port 8080 is not in use
lsof -i :8080

# Check firewall settings
# Check Docker network settings
docker network inspect bridge
```

## Performance

**Expected performance:**
- Download speed: ~20-30 MB/s (depends on network and HuggingFace bandwidth)
- Memory usage: ~200-500MB (bounded by parallel fetcher)
- Parallel chunks: 4-8 concurrent fetches
- Startup time: < 2 seconds

**Performance tips:**
- Use WARP-compatible image to avoid disabling/enabling WARP
- Multiple concurrent downloads are supported
- For high load, run multiple containers behind a load balancer

## Security

- Container runs as non-root user (UID 1000)
- HF_TOKEN is never logged or exposed in API responses
- WARP certificate is mounted read-only
- No persistent storage (stateless)
- Health checks use minimal permissions

## Updating

```bash
# Pull latest code
git pull

# Rebuild image
docker-compose build

# Recreate container
docker-compose down
docker-compose up -d
```

## Logs

```bash
# Follow logs
docker logs -f xet-proxy

# Last 100 lines
docker logs --tail 100 xet-proxy

# Since timestamp
docker logs --since 2024-01-01T00:00:00 xet-proxy
```

## Stopping and Removing

```bash
# Stop
docker-compose stop

# Stop and remove
docker-compose down

# Remove with volumes
docker-compose down -v

# Remove image
docker rmi xet-proxy:latest
```

## Advanced

### Custom Zig Version

```bash
# Build with different Zig version
docker build \
  --build-arg ZIG_VERSION=0.16.0-dev.2100+abcdef123 \
  -t xet-proxy:custom \
  -f Dockerfile .
```

### Multi-Stage Build Inspection

```bash
# Build only the builder stage
docker build --target builder -t xet-proxy-builder .

# Inspect builder
docker run -it xet-proxy-builder sh
```

### Kubernetes Deployment

See `kubernetes/` directory for example manifests (if available).

## Support

For issues and questions:
- GitHub Issues: https://github.com/yourusername/proxy-xet/issues
- XET Protocol Spec: https://jedisct1.github.io/draft-denis-xet/draft-denis-xet.html
