# AGENTS.md - Developer Guide for proxy-xet

This guide is for AI coding agents and developers working on the proxy-xet project, a pure Zig implementation of the XET protocol for efficient file storage through content-defined chunking and deduplication.

## Project Overview

This is a Zig implementation of the XET protocol for handling large files through content-defined chunking, compression, and deduplication. The codebase is designed to be cross-verified against the Rust reference implementation and follows the official XET protocol specification exactly.

## Latest Progress (Jan 11, 2026)

### ✅ FULLY WORKING - Production Ready!

#### Code Logic Verification ✅
- **All 106 unit tests passing** - Zig implementation verified
- **All 14 executables building successfully** - Clean compilation
- **Removed redundant `proxy_server.zig`** - Using Rust proxy instead

#### Deployment Status 🚀 PRODUCTION READY
- **Rust Proxy Server**: ✅ **TESTED AND WORKING**
  - Built with Axum (Rust async web framework)
  - Streams large files efficiently (35-45 MB/s)
  - Integrates with Zig `xet-download` CLI backend
  - **Successfully downloaded 7.73GB file in 3m38s**

- **Docker Container**: ✅ **TESTED AND WORKING**
  - Multi-stage build: Zig 0.16.0-dev.2145 + Rust 1.83
  - Multi-platform support: AMD64 (10MB) and ARM64 (36MB)
  - Cross-compilation from Apple Silicon to x86_64 servers
  - Health checks configured
  - Uses Alpine Linux for minimal size
  - **Note**: AMD64 builds work on ARM64 Macs via Docker buildx

#### Test Results (Jan 11, 2026)
```
✅ Health endpoint: {"status":"ok","version":"0.1.0"}
✅ File download: 7730M downloaded in 3m38s at 35.3 MB/s average (45.4 MB/s peak)
✅ Docker container: Runs successfully, all endpoints working
✅ XET hash resolution: Working correctly
```

### Quick Test Commands
```bash
# Build and run Rust proxy locally
cd proxy-rust && cargo build --release
export HF_TOKEN=your_token PORT=8080 ZIG_BIN_PATH=../zig-out/bin/xet-download
./target/release/xet-proxy

# Test endpoints
curl http://localhost:8080/health
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf -o test.gguf

# Build Docker container for AMD64 (for servers/private registry)
docker buildx build \
  --platform linux/amd64 \
  --file Dockerfile.proxy \
  --tag xet-proxy:latest \
  --load \
  .

# Or build for ARM64 (for Apple Silicon local development)
docker buildx build \
  --platform linux/arm64 \
  --file Dockerfile.proxy \
  --tag xet-proxy:latest \
  --load \
  .

# Run Docker container
export HF_TOKEN=your_token
docker run -p 8080:8080 -e HF_TOKEN=$HF_TOKEN xet-proxy:latest

# Test Docker container
curl http://localhost:8080/health
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf -o test.gguf
```

## Build System & Commands

### Build Commands
```bash
# Build everything (creates zig-out/bin/)
zig build

# Build with optimization
zig build -Doptimize=ReleaseFast

# Clean build artifacts
rm -rf zig-out .zig-cache
```

### Test Commands
```bash
# Run all tests (98 tests covering all components)
zig build test

# Run a specific test file
zig test src/chunking.zig --dep lz4 --mod lz4:lz4:$(zig build --build-file build.zig 2>&1 | grep lz4)

# Run tests with optimizations (faster execution)
zig build test -Doptimize=ReleaseFast

# Run tests with summary output
zig build test --summary all
```

### Running Single Tests
To run a single test in Zig, use the `--test-filter` option:

```bash
# Run a specific test by name
zig build test -- --summary all --test-filter "hash conversion"

# Run all tests in a specific file (requires building that test file directly)
zig test src/chunking.zig

# Run with verbose output
zig build test --verbose
```

### Build Commands

```bash
# Build the project (creates executable in zig-out/bin/)
zig build

# Build with optimization modes
zig build -Doptimize=ReleaseFast
zig build -Doptimize=ReleaseSmall
zig build -Doptimize=ReleaseSafe

# Run the main executable
zig build run

# Run benchmarks
zig build bench

# Run specific examples
zig build run-example-download        # Download model from HuggingFace
zig build run-example-parallel        # Parallel download example
zig build run-file-to-xorb            # Convert file to xorb format
```

### Running Tests

```bash
# Run all tests (98 tests across all modules)
zig build test

# Run tests in a specific file (build and run manually)
zig test src/chunking.zig --deps lz4 --mod lz4::$(zig build-obj --show-cache-dir)/o/lz4/lz4.zig

# Run a specific test by name (filter tests)
zig build test --summary all 2>&1 | grep "test_name"
```

Note: Zig doesn't have built-in single test filtering like Jest. To run a specific test, you need to:
1. Comment out other tests temporarily, or
2. Use `zig test src/specific_file.zig` for a single file

## Code Style Guidelines

### File Organization
- Each module should have a file-level doc comment (`//!`) describing its purpose
- Public API should be documented with doc comments (`///`)
- Tests live in the same file as the implementation they test
- Use `test` blocks at the bottom of each file

### Imports and Dependencies
- Standard library imports first: `const std = @import("std");`
- External dependencies second: `const lz4 = @import("lz4");`
- Internal imports last: `const constants = @import("constants.zig");`
- Import order: std library, external deps, internal modules
- Use explicit imports: `const Allocator = std.mem.Allocator;`

### Module Organization

```zig
//! Module docstring describing purpose and key features
//! Use autodoc style with examples if complex

const std = @import("std");
const builtin = @import("builtin");

// Constants first
pub const Something = struct { ... };

// Public API functions
pub fn publicFunction() void {}

// Private helpers
fn privateHelper() void {}

// Tests at the end
test "description of test" {
    // test code
}
```

### Types and Declarations

1. Use explicit types for clarity
2. Prefer `u8`, `u32`, `u64` over `usize` when size is protocol-defined
3. Use `@intCast()` for explicit conversions
4. Always check errors with `try` or `catch`
5. Use `errdefer` for cleanup in functions that can fail

### Naming Conventions

- **Functions**: `camelCase` (e.g., `chunkBuffer`, `computeDataHash`)
- **Types/Structs**: `PascalCase` (e.g., `ChunkBoundary`, `MerkleNode`)
- **Constants**: `PascalCase` for compile-time constants (e.g., `MaxChunkSize`)
- **Variables**: `snake_case` for locals and struct fields
- **Files**: `snake_case.zig` (e.g., `cas_client.zig`, `model_download.zig`)

### Memory Management

- Always use `errdefer` to clean up resources on error paths
- Explicitly pass allocators; never use global allocators
- Free all allocated memory with corresponding `deinit()` or `free()` calls
- Use `defer` statements immediately after allocation for cleanup
- Prefer stack allocation for small, fixed-size buffers

### Error Handling

- Use Zig's explicit error handling with `!` and `try`
- Define custom error sets for each module (e.g., `CompressionError`, `CasError`)
- Classify errors appropriately (retryable vs non-retryable for network operations)
- Use `errdefer` for cleanup in functions that allocate resources
- Propagate errors up the call stack; avoid silent failures

### Testing

- All modules must have comprehensive test coverage
- Use `test "description"` blocks for unit tests
- Test files can be standalone (`*_test.zig`) or inline
- Cross-verification tests exist in `verification_test.zig` and `bg4_verification_test.zig`
- Tests use `std.testing.allocator` for memory allocations
- All tests must pass before committing changes

### Memory Management

- Use `allocator` parameter for all dynamic allocations
- Implement `deinit()` methods for structs that own memory
- Use `errdefer` for cleanup in error paths
- Prefer `ArrayList` with `.empty` initialization for dynamic arrays
- Free resources in reverse order of allocation
- Use `defer` and `errdefer` consistently for cleanup

### Error Handling

- Use explicit error sets (e.g., `CompressionError`, `CasError`)
- Return errors using `!` syntax (e.g., `![]u8`, `!CompressionResult`)
- Use `try` for error propagation
- Use `catch` for specific error handling
- Use `errdefer` for cleanup on error paths
- Document error conditions in function comments
- Classify errors appropriately (retryable vs non-retryable for network operations)

### Naming Conventions

- **Types**: PascalCase (e.g., `ChunkBoundary`, `MerkleNode`, `CasClient`)
- **Functions**: camelCase (e.g., `computeDataHash`, `chunkBuffer`, `buildMerkleTree`)
- **Constants**: PascalCase for types, SCREAMING_SNAKE_CASE for compile-time constants
- **Variables**: snake_case (e.g., `chunk_start`, `file_hash`, `allocator`)
- **Files**: snake_case.zig (e.g., `cas_client.zig`, `model_download.zig`)
- **Test functions**: Descriptive names with spaces: `test "chunk boundary size calculation"`

### Error Handling

- Use Zig's error union types (`!Type`) for fallible operations
- Define module-specific error sets (e.g., `CompressionError`, `CasError`)
- Use `errdefer` for cleanup on error paths
- Classify errors appropriately (retryable vs non-retryable for network ops)
- Propagate errors with `try` unless handling is required

Example:
```zig
pub fn compress(allocator: Allocator, data: []const u8) !CompressionResult {
    const buffer = try allocator.alloc(u8, max_size);
    errdefer allocator.free(buffer);
    // ... compression logic
    return .{ .data = buffer, .type = .LZ4 };
}
```

### Memory Management

- Always use allocators passed as parameters (typically first parameter)
- Implement `deinit()` methods for structs that own memory
- Use `defer` and `errdefer` for cleanup
- Document memory ownership in function comments
- Prefer `ArrayList` over raw slices when dynamic growth is needed

Example:
```zig
pub const Token = struct {
    data: []u8,
    allocator: Allocator,
    
    pub fn deinit(self: *Token) void {
        self.allocator.free(self.data);
    }
};
```

### Documentation

- Use doc comments (`//!`) for module-level documentation at file start
- Use doc comments (`///`) for public API documentation
- Include examples in doc comments for complex functions
- Document memory ownership and lifetime expectations
- Note protocol compliance details where relevant

Example:
```zig
//! XET Protocol Compression: None, LZ4, ByteGrouping4LZ4

/// Compress data using the specified compression type.
/// Returns the compressed data and actual compression type used.
/// Caller owns returned memory and must free it.
pub fn compress(allocator: Allocator, data: []const u8, 
                compression_type: CompressionType) !CompressionResult
```

### Testing

- Write test blocks at the end of each module
- Use descriptive test names with spaces (e.g., `test "hash conversion - roundtrip"`)
- Test both success and error cases
- Use `std.testing.allocator` for memory leak detection
- Always `defer` cleanup in tests
- Test determinism where applicable

Example:
```zig
test "chunker produces deterministic results" {
    const allocator = std.testing.allocator;
    
    var data: [1000]u8 = undefined;
    var chunks1 = try chunkBuffer(allocator, &data);
    defer chunks1.deinit(allocator);
    
    var chunks2 = try chunkBuffer(allocator, &data);
    defer chunks2.deinit(allocator);
    
    try std.testing.expectEqual(chunks1.items.len, chunks2.items.len);
}
```

### Performance Considerations

- Use `inline` for hot path functions when appropriate
- Prefer stack allocation for small buffers (array, not slice)
- Use thread pools for parallel operations (see `parallel_fetcher.zig`)
- Minimize allocations in tight loops
- Use `@intCast` when converting between integer types

### Platform Support

- Code should work on non-WASM targets by default
- Use conditional compilation for WASM limitations:
  ```zig
  pub const has_network_support = builtin.target.os.tag != .wasi;
  pub const cas_client = if (has_network_support) @import("cas_client.zig") else struct {};
  ```
- Network and threading features are not available on WASM

### Constants and Magic Numbers

- Define all protocol constants in `constants.zig`
- Use descriptive constant names (e.g., `MinChunkSize`, `MaxChunkSize`)
- Include units in variable names when relevant (e.g., `timeout_ms`)

### Error Messages and Logging

- Use `std.debug.print` sparingly, primarily for debugging
- Prefer returning errors over printing
- Include context in error returns where helpful

## Project-Specific Notes

### XET Protocol Compliance

This codebase implements the official XET protocol specification. When making changes:

- Maintain byte-for-byte compatibility with the Rust reference implementation
- Do not modify chunking, hashing, or compression algorithms without protocol justification
- Run cross-verification tests (`verification_test.zig`, `bg4_verification_test.zig`) after changes
- Consult the protocol spec: https://jedisct1.github.io/draft-denis-xet/draft-denis-xet.html

### Key Modules

- `constants.zig`: Protocol constants (GearHashTable, BLAKE3 keys, sizes)
- `chunking.zig`: Gearhash content-defined chunking (8KB-128KB chunks)
- `hashing.zig`: BLAKE3 with 4 domain-separation keys + Merkle trees
- `compression.zig`: LZ4, ByteGrouping4LZ4, FullBitsliceLZ4 compression
- `xorb.zig`: Xorb format serialization/deserialization
- `shard.zig`: MDB shard format I/O
- `cas_client.zig`: HTTP CAS API client (network-dependent)
- `model_download.zig`: High-level HuggingFace download API
- `parallel_fetcher.zig`: Thread pool for parallel chunk fetching
- `reconstruction.zig`: File reconstruction from xorb terms

### Common Patterns

1. **Hash Conversion**: Use `hashToHex()` and `hexToHash()` from `hashing.zig`
2. **HTTP Requests**: Use `std.http.Client` with proper auth headers
3. **JSON Parsing**: Use `std.json.parseFromSlice()` with `defer parsed.deinit()`
4. **Compression**: Always check if compression actually reduces size, fall back to `.None`

### Gotchas

- Zig uses 0-based indexing but some protocol fields are 1-based (check spec)
- HTTP byte ranges are inclusive on both ends in Range headers
- BLAKE3 requires domain separation keys for different hash types
- Chunk boundaries depend on first chunk skipping MIN-65 bytes
- Memory limits: 80MB for xorb/shard downloads (protocol max is 64MB + overhead)

## Known Issues

### TLS Certificate Issues with VPN/Proxy Software

If you encounter TLS certificate errors like `CertificateSignatureNamedCurveUnsupported` when connecting to HuggingFace's API, this is likely caused by VPN or proxy software (like Cloudflare WARP) intercepting HTTPS connections.

**Error**: `TlsInitializationFailed` or `CertificateSignatureNamedCurveUnsupported`

**Solution**: Disable VPN/proxy software (e.g., Cloudflare WARP) temporarily, or configure it to exclude HuggingFace domains.

**Example working download**:
```bash
# Download a specific file (requires HF_TOKEN environment variable)
HF_TOKEN=hf_xxx zig build run-example-download -- jedisct1/MiMo-7B-RL-GGUF MiMo-7B-RL-Q8_0.gguf

# Example output:
# Download complete!
#   Time: 321.55s
#   Size: 8106510944 bytes (7.55 GB)
#   Speed: 24.04 MB/s
```

The Zig TLS implementation works correctly with HuggingFace's certificates when no proxy intercepts the connection.

## Docker Deployment

The XET Proxy Server can be deployed in Docker with multi-platform support.

### Multi-Platform Docker Builds

The project supports building Docker images for multiple platforms using Docker buildx:

**Available platforms:**
- `linux/amd64` (x86_64) - Standard server architecture
- `linux/arm64` (aarch64) - Apple Silicon, ARM servers

**Current image sizes:**
- AMD64: ~10.4 MB (optimized Alpine-based)
- ARM64: ~36.6 MB (includes additional libraries)

### Building for AMD64 (Production Deployment)

Build for AMD64 (x86_64) to deploy on standard servers or push to private registries:

```bash
docker buildx build \
  --platform linux/amd64 \
  --file Dockerfile.proxy \
  --tag xet-proxy:latest \
  --load \
  .
```

**Build details:**
- Build time: ~90-120 seconds
- Cross-compiles from ARM64 Mac using QEMU emulation
- Multi-stage build: Zig builder + Rust builder + Alpine runtime
- Result: Statically-linked binaries in minimal Alpine image (~10MB)

### Building for ARM64 (Local Development)

Build for ARM64 (Apple Silicon Macs, ARM servers):

```bash
docker buildx build \
  --platform linux/arm64 \
  --file Dockerfile.proxy \
  --tag xet-proxy:latest \
  --load \
  .
```

**Note:** ARM64 builds are faster on Apple Silicon (native compilation, no emulation)

### Verify Image Architecture

```bash
# Check platform
docker image inspect xet-proxy:latest --format '{{.Architecture}}'

# Full details
docker image inspect xet-proxy:latest --format '{{.Architecture}} {{.Os}} {{.Size}}'
```

### Deployment to Private Registry

#### Tag and Push to Registry

```bash
# Tag for your private registry
docker tag xet-proxy:latest registry.example.com/xet-proxy:latest
docker tag xet-proxy:latest registry.example.com/xet-proxy:v0.1.0

# Push to registry
docker push registry.example.com/xet-proxy:latest
docker push registry.example.com/xet-proxy:v0.1.0
```

#### Export to Tar File (Airgapped Deployment)

```bash
# Save AMD64 image to tar file
docker save xet-proxy:latest -o xet-proxy-amd64.tar

# Transfer xet-proxy-amd64.tar to target server, then:
docker load -i xet-proxy-amd64.tar
```

### Docker Image Cleanup

Clean up unused images while preserving specific ones:

```bash
# Check current Docker disk usage
docker system df

# Remove dangling images (untagged)
docker image prune -f

# Remove all unused images
docker image prune -a -f

# Clean build cache (reclaim space)
docker builder prune -f

# Remove all images except specific ones by ID
docker images --format "{{.Repository}}:{{.Tag}} {{.ID}}" | \
  grep -v "IMAGE_ID_TO_KEEP" | \
  awk '{print $2}' | \
  xargs docker rmi -f
```

### Using Docker Compose

For quick local testing with Docker Compose:

```bash
# Build and run
export HF_TOKEN=hf_xxx
docker-compose -f docker-compose.proxy.yml up -d

# Test
curl http://localhost:8080/health

# Stop and remove
docker-compose -f docker-compose.proxy.yml down
```

### WARP-Compatible Docker (Optional)

For environments where Cloudflare WARP cannot be disabled:

```bash
# Extract WARP certificate (one-time)
./scripts/extract-warp-cert.sh

# Build and run
export HF_TOKEN=hf_xxx
docker-compose -f docker-compose.warp.yml up -d

# Test
curl http://localhost:8080/health
```

### Proxy Server Endpoints

- `GET /health` - Health check
- `GET /download/:repo_id/:file_path` - Download by repo and path
- `GET /download-hash/:xet_hash_hex` - Download by XET hash
- `GET /` - Usage instructions

### Example Usage

```bash
# Download a file via proxy
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf \
  -o model.gguf

# Download by hash
curl http://localhost:8080/download-hash/ef62b7509a2c...5bd -o model.safetensors
```

For detailed Docker deployment instructions, see [DOCKER.md](DOCKER.md).