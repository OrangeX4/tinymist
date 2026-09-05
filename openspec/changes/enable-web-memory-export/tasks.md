- [x] Separate export scheduling and filesystem writes from shared byte generation.
- [x] Build web exports without system, lock or open capabilities.
- [x] Verify native exports and the feature matrix.
- [x] Verify actual WASM metadata queries and PDF/SVG output.

Validation: all eight native export tests, the full feature-testing script (including
wasm32 web+export), and workspace formatting pass. Browser tests in the consuming Tylina
application verify real Query/PDF/SVG/selected-page PNG responses on Unicode virtual
paths, rejected filesystem writes, unchanged published source, and compiled PDFPC notes.
