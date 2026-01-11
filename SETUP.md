# Setup Guide

## Quick Setup (2 minutes)

### 1. Get Your HuggingFace Token

1. Go to https://huggingface.co/settings/tokens
2. Click "New token"
3. Select "Read" access
4. Copy the token (starts with `hf_`)

### 2. Configure Token

**Option A: Environment Variable (Recommended)**

Add to your shell profile:

```bash
# For macOS/Linux with zsh (most Macs)
echo 'export HF_TOKEN="hf_your_actual_token_here"' >> ~/.zshrc
source ~/.zshrc

# For Linux with bash
echo 'export HF_TOKEN="hf_your_actual_token_here"' >> ~/.bashrc
source ~/.bashrc
```

**Option B: .env File (Project-specific)**

```bash
# Copy the example file
cp .env.example .env

# Edit with your token
echo 'HF_TOKEN=hf_your_actual_token_here' > .env
```

The .env file is already in `.gitignore` and will never be committed to git.

### 3. Run the Proxy

**With Docker:**

```bash
# Using environment variable
docker run -p 8080:8080 -e HF_TOKEN=$HF_TOKEN xet-proxy:latest

# Or with docker-compose (reads .env automatically)
docker-compose -f docker-compose.proxy.yml up
```

**Without Docker:**

```bash
# Build
zig build -Doptimize=ReleaseFast
cd proxy-rust && cargo build --release && cd ..

# Run
export ZIG_BIN_PATH=./zig-out/bin/xet-download
./proxy-rust/target/release/xet-proxy
```

### 4. Test It

```bash
# Health check
curl http://localhost:8080/health

# Download a file
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/README.md
```

## Security Best Practices

### ✅ DO:
- Store token in environment variables or .env file
- Add .env to .gitignore (already done)
- Use different tokens for dev/prod
- Regenerate tokens periodically

### ❌ DON'T:
- Commit tokens to git
- Share tokens in chat/email
- Use tokens in code/documentation
- Store tokens in plain text files tracked by git

## Deployment Environments

### Local Development

Use .env file:
```bash
cp .env.example .env
# Edit .env with your token
docker-compose up
```

### Production Server

Set environment variable:
```bash
export HF_TOKEN=hf_xxx
docker run -d -p 8080:8080 -e HF_TOKEN=$HF_TOKEN xet-proxy:latest
```

### CI/CD (GitHub Actions)

Add as repository secret:
1. Go to Settings → Secrets and variables → Actions
2. Add `HF_TOKEN` as a secret
3. Use in workflow:

```yaml
env:
  HF_TOKEN: ${{ secrets.HF_TOKEN }}
```

### Kubernetes

Create secret:
```bash
kubectl create secret generic hf-token \
  --from-literal=HF_TOKEN=hf_your_token
```

Use in deployment:
```yaml
env:
  - name: HF_TOKEN
    valueFrom:
      secretKeyRef:
        name: hf-token
        key: HF_TOKEN
```

## Troubleshooting

### "HF_TOKEN not set" error

```bash
# Check if token is set
echo $HF_TOKEN

# If empty, set it
export HF_TOKEN=hf_your_token

# Or source your shell profile
source ~/.zshrc  # or ~/.bashrc
```

### Docker can't find token

```bash
# Pass explicitly
docker run -e HF_TOKEN=$HF_TOKEN xet-proxy:latest

# Or use .env file with docker-compose
docker-compose --env-file .env up
```

### Token expired or invalid

1. Go to https://huggingface.co/settings/tokens
2. Check token status
3. Generate new token if needed
4. Update your environment variable or .env file

## Next Steps

- [README.md](README.md) - Project overview
- [DOCKER.md](DOCKER.md) - Docker deployment guide
- [AGENTS.md](AGENTS.md) - Developer guide
