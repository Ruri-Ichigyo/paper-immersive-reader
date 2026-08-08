# Vendored pdf.js Assets

This directory holds vendored pdf.js worker and runtime assets for
browser-side PDF text extraction.

Place the following files here before running the web app:

- `pdf.min.js`
- `pdf.worker.min.js`

Download from: https://github.com/mozilla/pdf.js/releases

Choose the `pdfjs-X.Y.Z-dist` zip, then copy `build/pdf.min.js` and
`build/pdf.worker.min.js` from the archive into this directory.

The files are copied to the Trunk dist output via `data-trunk rel="copy-file"`
links in `index.html` and loaded at runtime.
