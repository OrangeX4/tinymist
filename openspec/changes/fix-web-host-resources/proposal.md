## Why

Web LSP file conversion currently treats percent-encoded URI text as a filesystem path.
Imports of Unicode filenames fail, and filenames containing `#`, `?`, or `%` cannot round-trip.
Hosts also need to publish fonts and verified package bytes after asynchronous downloads.

## What Changes

- Encode virtual path components using the URL library and decode URI paths using percent-encoding.
- Expose Rust host methods for replacing fonts and publishing filesystem changes to an initialized Web server.
- Preserve open documents and use the existing project interruption boundary for invalidation.
- Include memory files in analysis workspace enumeration.
- Advance VFS revisions for new files and accumulate invalidation across filesystem batches.

## Impact

WASM LSP hosts; native file URI conversion remains platform-owned.
