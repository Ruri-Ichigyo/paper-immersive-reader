#!/bin/sh
set -eu

pdf_path="${1:-}"

if [ -z "$pdf_path" ]; then
  echo "usage: scripts/run_paddle_ocr.sh <pdf-path>" >&2
  exit 64
fi

echo "run_paddle_ocr.sh is a placeholder adapter. Replace it with a real PaddleOCR command that writes recognized text to stdout for: $pdf_path" >&2
exit 1
