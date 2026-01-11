//! Simple CLI wrapper for downloading XET files
//! This is called by the Rust proxy server to handle actual downloads

const std = @import("std");
const xet = @import("xet");

pub fn main(init: std.process.Init) !void {
    const allocator = init.gpa;
    const io = init.io;
    const environ = init.minimal.environ;

    var args_iter = std.process.Args.Iterator.init(init.minimal.args);
    var args: std.ArrayList([]const u8) = .empty;
    defer args.deinit(allocator);

    while (args_iter.next()) |arg| {
        try args.append(allocator, arg);
    }

    // Usage: download_cli <repo_id> [filename_or_hash]
    if (args.items.len < 2) {
        var stderr_buffer: [256]u8 = undefined;
        var stderr_writer = std.Io.File.stderr().writer(io, &stderr_buffer);
        try stderr_writer.interface.writeAll("Usage: download_cli <repo_id> [filename_or_hash]\n");
        try stderr_writer.interface.flush();
        return error.InvalidArgs;
    }

    const repo_id = args.items[1];
    const file_or_hash = if (args.items.len > 2) args.items[2] else null;

    // Get HF token
    const hf_token = try std.process.Environ.getAlloc(environ, allocator, "HF_TOKEN");
    defer allocator.free(hf_token);

    // If file_or_hash looks like a hash (64 hex chars), download by hash
    if (file_or_hash) |foh| {
        if (foh.len == 64 and isHex(foh)) {
            try downloadByHash(allocator, io, environ, foh, hf_token);
            return;
        }
    }

    // Otherwise, list files
    try listFiles(allocator, io, environ, repo_id, hf_token, file_or_hash);
}

fn isHex(s: []const u8) bool {
    for (s) |c| {
        if (!std.ascii.isHex(c)) return false;
    }
    return true;
}

fn listFiles(
    allocator: std.mem.Allocator,
    io: std.Io,
    environ: std.process.Environ,
    repo_id: []const u8,
    hf_token: []const u8,
    filename: ?[]const u8,
) !void {
    var file_list = try xet.model_download.listFiles(
        allocator,
        io,
        environ,
        repo_id,
        "model",
        "main",
        hf_token,
    );
    defer file_list.deinit();

    var stdout_buffer: [4096]u8 = undefined;
    var stdout_writer = std.Io.File.stdout().writer(io, &stdout_buffer);
    defer stdout_writer.interface.flush() catch {};
    const stdout = &stdout_writer.interface;

    // If filename specified, find and download it
    if (filename) |name| {
        const file_info = file_list.findFile(name) orelse {
            try stdout.writeAll("File not found\n");
            return error.FileNotFound;
        };

        if (file_info.xet_hash == null) {
            try stdout.writeAll("File is not XET-enabled\n");
            return error.NotXetFile;
        }

        try downloadByHash(allocator, io, environ, file_info.xet_hash.?, hf_token);
        return;
    }

    // Otherwise just list files with XET hashes
    for (file_list.files) |file| {
        if (file.xet_hash) |hash| {
            const size_mb = @as(f64, @floatFromInt(file.size)) / (1024.0 * 1024.0);
            try stdout.print("{s} - {d:.2} MB - xetHash: {s}\n", .{
                file.path,
                size_mb,
                hash,
            });
        }
    }
    try stdout.flush();
}

fn downloadByHash(
    allocator: std.mem.Allocator,
    io: std.Io,
    environ: std.process.Environ,
    hash_hex: []const u8,
    hf_token: []const u8,
) !void {
    _ = try xet.cas_client.apiHexToHash(hash_hex);

    const config = xet.model_download.DownloadConfig{
        .repo_id = "jedisct1/MiMo-7B-RL-GGUF", // Temporary repo for token
        .repo_type = "model",
        .revision = "main",
        .file_hash_hex = hash_hex,
        .hf_token = hf_token,
    };

    // Stream to stdout
    var stdout_buffer: [8192]u8 = undefined;
    var stdout_writer = std.Io.File.stdout().writer(io, &stdout_buffer);
    defer stdout_writer.interface.flush() catch {};

    try xet.model_download.downloadModelToWriter(
        allocator,
        io,
        environ,
        config,
        &stdout_writer.interface,
    );
}
