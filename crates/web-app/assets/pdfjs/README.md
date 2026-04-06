This directory is reserved for vendored `pdf.js` runtime assets.

The current MVP batch keeps the browser extraction seam behind `crates/web-app/src/import/pdf_bridge.rs`.
Future browser wiring should place the checked-in `pdf.js` worker/runtime files here without introducing a second package manager.
