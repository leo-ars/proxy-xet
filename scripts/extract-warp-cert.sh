#!/bin/bash
# Extract Cloudflare WARP certificate from macOS keychain
#
# Usage: ./scripts/extract-warp-cert.sh [output_file]
# Default output: cloudflare-warp.crt

set -e

CERT_NAME="Cloudflare"
OUTPUT_FILE="${1:-cloudflare-warp.crt}"

echo "🔍 Searching for Cloudflare WARP certificate in keychain..."

# Extract certificate from keychain
if security find-certificate -c "$CERT_NAME" -p > "$OUTPUT_FILE" 2>/dev/null; then
    if [ -s "$OUTPUT_FILE" ]; then
        echo "✅ Certificate extracted to: $OUTPUT_FILE"
        echo ""
        echo "📋 Certificate details:"
        openssl x509 -in "$OUTPUT_FILE" -noout -subject -issuer -dates 2>/dev/null || {
            echo "⚠️  Warning: Could not parse certificate details"
        }
        echo ""
        echo "✅ Certificate is ready to use with Docker"
        echo "   Mount it with: -v ./$(basename "$OUTPUT_FILE"):/usr/local/share/ca-certificates/cloudflare-warp.crt:ro"
        exit 0
    else
        echo "❌ Certificate file is empty"
        rm -f "$OUTPUT_FILE"
        exit 1
    fi
else
    echo "❌ Failed to extract Cloudflare certificate from keychain"
    echo ""
    echo "Possible reasons:"
    echo "  1. Cloudflare WARP is not installed"
    echo "  2. WARP certificate is not in system keychain"
    echo "  3. Certificate name has changed"
    echo ""
    echo "To check available certificates:"
    echo "  security find-certificate -a -p | openssl x509 -noout -subject"
    rm -f "$OUTPUT_FILE"
    exit 1
fi
