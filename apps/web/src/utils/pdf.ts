import * as pdfjs from "pdfjs-dist";

pdfjs.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url,
).toString();

interface PdfTextItem {
  str: string;
  transform: number[]; // [scaleX, skewY, skewX, scaleY, translateX, translateY]
  fontName?: string;
  width?: number;
  height?: number;
  hasEOL?: boolean;
}

interface TextLine {
  y: number;
  x0: number;
  x1: number;
  text: string;
  fontSize: number;
  page: number;
  items: PdfTextItem[];
}

/** Extract text from a PDF and convert to HTML with layout-aware paragraph detection. */
export async function extractPdfText(file: File): Promise<string> {
  const data = await file.arrayBuffer();
  const doc = await pdfjs.getDocument({ data }).promise;

  // Collect all text items with their positions and font sizes
  const allItems: (PdfTextItem & { page: number })[] = [];
  for (let i = 1; i <= doc.numPages; i++) {
    const page = await doc.getPage(i);
    const content = await page.getTextContent();
    for (const item of content.items as unknown as PdfTextItem[]) {
      if (typeof item.str !== "string") continue;
      allItems.push({ ...item, page: i });
    }
  }

  // Group items into lines by Y position
  const lines = groupIntoLines(allItems);

  // Detect column layout and page boundaries
  const paragraphs = detectParagraphs(lines);

  return paragraphs.join("\n\n");
}

/** Group text items into lines by their Y position. */
function groupIntoLines(items: (PdfTextItem & { page: number })[]): TextLine[] {
  const lines: TextLine[] = [];
  for (const item of items) {
    if (item.str.trim() === "") continue;
    const y = Math.round(item.transform[5] ?? 0);
    const x = Math.round(item.transform[4] ?? 0);
    const fontSize = Math.abs(item.transform[3]) || Math.abs(item.transform[0]) || 12;

    // Find an existing line on the same page at roughly the same Y
    let existing = lines.find(
      (l) => l.page === item.page && Math.abs(l.y - y) <= 3,
    );

    if (existing) {
      existing.text += " " + item.str;
      existing.x0 = Math.min(existing.x0, x);
      existing.x1 = Math.max(existing.x1, x + (item.width ?? item.str.length * fontSize * 0.5));
      existing.items.push(item);
      // Use the most common font size
      existing.fontSize = medianFontSize(existing.items);
    } else {
      lines.push({
        y,
        x0: x,
        x1: x + (item.width ?? item.str.length * fontSize * 0.5),
        text: item.str,
        fontSize,
        page: item.page,
        items: [item],
      });
    }
  }

  // Sort lines: by page, then by Y descending (PDF coords are bottom-up)
  lines.sort((a, b) => (a.page - b.page) || (b.y - a.y));
  return lines;
}

function medianFontSize(items: PdfTextItem[]): number {
  const sizes = items
    .map((i) => Math.abs(i.transform[3]) || Math.abs(i.transform[0]) || 12)
    .sort((a, b) => a - b);
  return sizes[Math.floor(sizes.length / 2)] ?? 12;
}

/** Detect paragraphs from a sequence of lines. */
function detectParagraphs(lines: TextLine[]): string[] {
  if (lines.length === 0) return [];

  // Compute page width from line extents
  const pageWidths = new Map<number, number>();
  for (const line of lines) {
    const w = line.x1;
    pageWidths.set(line.page, Math.max(pageWidths.get(line.page) ?? 0, w));
  }

  // Compute median font size (body text baseline)
  const fontSizes = lines.map((l) => l.fontSize).sort((a, b) => a - b);
  const bodyFontSize = fontSizes[Math.floor(fontSizes.length / 2)] ?? 12;

  // Heading threshold: font size >= 125% of body
  const headingThreshold = bodyFontSize * 1.25;

  // Compute typical line spacing to detect paragraph gaps
  const spacings: number[] = [];
  for (let i = 1; i < lines.length; i++) {
    const prev = lines[i - 1]!;
    const curr = lines[i]!;
    if (prev.page === curr.page) {
      spacings.push(Math.abs(prev.y - curr.y));
    }
  }
  spacings.sort((a, b) => a - b);
  const typicalSpacing = spacings[Math.floor(spacings.length / 2)] ?? bodyFontSize * 1.5;
  // A gap > 1.8× typical spacing is a paragraph break
  const paragraphGapThreshold = typicalSpacing * 1.8;

  const paragraphs: string[] = [];
  let current = "";

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    const prev = i > 0 ? lines[i - 1]! : null;
    const next = i < lines.length - 1 ? lines[i + 1]! : null;

    // Page break → flush
    if (prev && line.page !== prev.page) {
      if (current.trim()) {
        paragraphs.push(current.trim());
        current = "";
      }
    }

    // Detect heading
    const isHeading = line.fontSize >= headingThreshold && isHeadingLike(line.text);

    if (isHeading) {
      if (current.trim()) {
        paragraphs.push(current.trim());
        current = "";
      }
      paragraphs.push(`__H2__${line.text}`);
      continue;
    }

    // Detect paragraph break (large Y gap, or previous line ended a sentence and this starts fresh)
    const yGap = prev && prev.page === line.page ? Math.abs(prev.y - line.y) : 0;
    const isLargeGap = yGap > paragraphGapThreshold;

    // Check if current paragraph ends with sentence-ending punctuation
    const currentEndsSentence = /[.!?。！？]\s*$/.test(current);
    const nextLineStartsFresh = /^[A-Z一-鿿]/.test(line.text);

    // Also detect: if current line is short and next line is much longer, current is probably a heading
    if (
      current.length < 60 &&
      !current.endsWith(".") &&
      !current.endsWith(",") &&
      next &&
      next.text.length > current.length * 1.5 &&
      next.page === line.page
    ) {
      // Current looks like a heading, but we already handled large-font headings above
      // Just flush if current ends a sentence
    }

    if (isLargeGap || (currentEndsSentence && nextLineStartsFresh && current.length > 40)) {
      if (current.trim()) {
        paragraphs.push(current.trim());
        current = "";
      }
    }

    // Merge line into current paragraph (strip internal newlines, join with space)
    const cleanLine = line.text.replace(/\s+/g, " ").trim();
    if (cleanLine) {
      current = current ? `${current} ${cleanLine}` : cleanLine;
    }
  }

  if (current.trim()) {
    paragraphs.push(current.trim());
  }

  return paragraphs;
}

function isHeadingLike(text: string): boolean {
  const trimmed = text.trim();
  if (trimmed.length === 0 || trimmed.length > 120) return false;
  // All caps (at least 3 letters)
  if (trimmed.length >= 3 && trimmed === trimmed.toUpperCase() && /[A-Z]/.test(trimmed)) return true;
  // Starts with "Chapter", "Section", "Part", numbered headings
  if (/^(chapter|section|part|abstract|references|acknowledgment|introduction|conclusion)\b/i.test(trimmed)) return true;
  if (/^\d+(\.\d+)*\s+[A-Z]/.test(trimmed)) return true;
  // Short and doesn't end with sentence punctuation
  if (trimmed.length < 60 && !/[.!?。！？]$/.test(trimmed) && !trimmed.endsWith(",")) return true;
  return false;
}
