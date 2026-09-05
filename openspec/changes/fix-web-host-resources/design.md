## Decisions

Web virtual paths are UTF-8 absolute paths. URL path segments encode reserved characters exactly once.
The browser conversion helpers are also exercised by a focused native unit test.

Font and filesystem publication use existing `Interrupt::Font` and `Interrupt::Fs` operations.
The configuration font cache receives the same resolver so a project reload does not revert host fonts.
The host owns download integrity and archive validation; these setters do not fetch or parse archives.

VFS revisions cover file inventory as well as previously read bytes. New memory files and notified resources
must invalidate cached world snapshots. Invalidation accumulates within a batch: an unchanged trailing file
cannot clear an earlier change. Identical repeated bytes do not advance the revision.

Analysis workspace enumeration includes shadow files, allowing references and rename to find importing
documents that exist only in editor memory.

## Validation

Tylina's real WASM browser fixture exercises cross-file imports with Unicode and reserved characters,
font catalog queries, semantic queries, and diagnostics after editing.
