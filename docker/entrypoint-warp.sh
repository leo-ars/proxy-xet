#!/bin/sh
# Entrypoint script for WARP-compatible Docker image
# Handles Cloudflare WARP certificate injection

set -e

WARP_CERT_PATH="/usr/local/share/ca-certificates/cloudflare-warp.crt"

# Check if WARP certificate is mounted
if [ -f "$WARP_CERT_PATH" ]; then
    echo "🔒 Cloudflare WARP certificate detected, updating CA trust store..."
    
    # Update CA certificates (requires root privileges)
    update-ca-certificates 2>&1 | grep -v "WARNING" || true
    
    echo "✅ CA certificates updated successfully"
    echo "   WARP certificate is now trusted by the system"
else
    echo "ℹ️  No WARP certificate found at $WARP_CERT_PATH"
    echo "   Running in standard mode (WARP should be disabled on host)"
fi

# Drop privileges to xet user and execute main command
echo "🚀 Starting XET Proxy Server (Rust)..."
echo "   Zig CLI: $ZIG_BIN_PATH"
exec su-exec xet "$@"
