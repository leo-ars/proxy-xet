# Minimal image with CA certificates
FROM scratch

# Copy CA certificate bundle from host
COPY ca-bundle.crt /etc/ssl/certs/ca-certificates.crt

# Copy pre-built Linux binaries (statically linked)
COPY zig-out/bin/xet-download /usr/local/bin/xet-download
COPY proxy-rust/target/aarch64-unknown-linux-musl/release/xet-proxy /usr/local/bin/xet-proxy

EXPOSE 8080

ENV ZIG_BIN_PATH=/usr/local/bin/xet-download
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

CMD ["/usr/local/bin/xet-proxy"]
