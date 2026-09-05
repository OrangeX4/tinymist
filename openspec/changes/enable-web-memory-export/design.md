## Design

Export bytes continue to use Tinymist's existing document compilation and task types.
Extract filesystem writing, computation scheduling and tests from the mixed export
module. Native hosts retain the existing Tokio/Rayon scheduling; browser hosts run
the same byte computation on their owning worker. Browser writes fail explicitly.

Project lock updates remain a lock-feature capability. Markdown export reads the
published memory source on hosts without filesystem support. Memory export keeps the
normal response envelope and base64 payloads. Native hosts retain suggested output
paths; browser exports return null paths because they do not designate disk files.

## Verification

Check the feature matrix, native export tests, and actual WASM query/PDF/SVG responses.
The consuming Web presenter verifies metadata against real compiled physical pages.
