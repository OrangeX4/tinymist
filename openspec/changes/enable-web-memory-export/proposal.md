## Why

Web language clients cannot query compiled metadata or export documents because the
export feature currently requires native filesystem and thread-pool support.

## What Changes

- Share the existing in-memory export implementation with browser builds.
- Keep filesystem writes and project-lock updates behind their platform features.
- Run browser export computation synchronously within the calling Web Worker.
- Preserve native export output, physical page numbering and output path patterns.

## Impact

The `web,export` feature combination supports real query/PDF/SVG exports without
enabling filesystem access or creating native threads.
