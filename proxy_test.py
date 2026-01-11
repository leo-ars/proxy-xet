#!/usr/bin/env python3
"""
Simple HTTP proxy test server for XET
This wraps the Zig CLI to test functionality without Docker
"""

import os
import subprocess
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

PORT = 8080
ZIG_BIN = "./zig-out/bin/xet-download"
HF_TOKEN = os.environ.get("HF_TOKEN", "")

class XETProxyHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path
        
        if path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"status":"ok","version":"0.1.0"}')
            return
        
        if path == "/":
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.end_headers()
            html = b"""
            <h1>XET Proxy Test Server</h1>
            <p>Endpoints:</p>
            <ul>
                <li>GET /health - Health check</li>
                <li>GET /download/owner/repo/file - Download file</li>
            </ul>
            """
            self.wfile.write(html)
            return
        
        if path.startswith("/download/"):
            # Parse: /download/owner/repo/file...
            parts = path[10:].split("/", 2)
            if len(parts) < 3:
                self.send_error(400, "Invalid path")
                return
            
            owner, repo, filename = parts
            repo_id = f"{owner}/{repo}"
            
            print(f"Downloading: {repo_id} / {filename}")
            
            try:
                # Run Zig CLI
                env = os.environ.copy()
                env["HF_TOKEN"] = HF_TOKEN
                
                # First list files to get hash
                proc = subprocess.Popen(
                    [ZIG_BIN, repo_id],
                    env=env,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                stdout, stderr = proc.communicate(timeout=30)
                
                if proc.returncode != 0:
                    self.send_error(500, f"Failed to list files: {stderr.decode()}")
                    return
                
                # Parse output to find hash
                xet_hash = None
                for line in stdout.decode().split("\n"):
                    if filename in line and "xetHash:" in line:
                        xet_hash = line.split("xetHash:")[1].strip()
                        break
                
                if not xet_hash:
                    self.send_error(404, f"File {filename} not found")
                    return
                
                print(f"Found hash: {xet_hash}")
                
                # Now download by hash
                proc = subprocess.Popen(
                    [ZIG_BIN, repo_id, xet_hash],
                    env=env,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                
                self.send_response(200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Disposition", f'attachment; filename="{filename}"')
                self.end_headers()
                
                # Stream output
                if proc.stdout:
                    while True:
                        chunk = proc.stdout.read(8192)
                        if not chunk:
                            break
                        self.wfile.write(chunk)
                
                proc.wait()
                
                if proc.returncode != 0 and proc.stderr:
                    print(f"Download failed: {proc.stderr.read().decode()}")
                
            except Exception as e:
                self.send_error(500, str(e))
                return
        else:
            self.send_error(404, "Not found")
    
    def log_message(self, format, *args):
        print(f"[{self.address_string()}] {format % args}")

if __name__ == "__main__":
    if not HF_TOKEN:
        print("ERROR: HF_TOKEN environment variable not set")
        sys.exit(1)
    
    if not os.path.exists(ZIG_BIN):
        print(f"ERROR: Zig binary not found at {ZIG_BIN}")
        print("Run: zig build -Doptimize=ReleaseFast")
        sys.exit(1)
    
    print("=" * 60)
    print("XET Proxy Test Server")
    print("=" * 60)
    print(f"Listening on: http://localhost:{PORT}")
    print(f"Zig CLI: {ZIG_BIN}")
    print(f"HF Token: {HF_TOKEN[:10]}...")
    print()
    print("Endpoints:")
    print("  GET /health")
    print("  GET /download/:owner/:repo/*file")
    print()
    print("Press Ctrl+C to stop")
    print("=" * 60)
    
    server = HTTPServer(("0.0.0.0", PORT), XETProxyHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()
