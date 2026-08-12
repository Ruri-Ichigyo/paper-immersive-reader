// Text extraction and segmentation helpers

/** Escape HTML special characters to prevent XSS. */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

/** Check if a line looks like a heading (short, possibly all caps or ends with colon). */
function isHeading(line: string): boolean {
  const trimmed = line.trim();
  if (trimmed.length === 0 || trimmed.length > 80) return false;
  // All caps (at least 3 chars)
  if (trimmed.length >= 3 && trimmed === trimmed.toUpperCase() && /[A-Z]/.test(trimmed)) return true;
  // Ends with colon and is short
  if (trimmed.endsWith(":") && trimmed.length < 60) return true;
  // Starts with "Chapter", "Section", "Part", numbered headings
  if (/^(chapter|section|part)\s+\d/i.test(trimmed)) return true;
  if (/^\d+(\.\d+)*\s+[A-Z]/.test(trimmed) && trimmed.length < 80) return true;
  return false;
}

/** Check if a line looks like a list item. */
function isListItem(line: string): boolean {
  const trimmed = line.trim();
  return /^[-*•]\s/.test(trimmed) || /^\d+[.)]\s/.test(trimmed);
}

/** Convert text to an array of HTML paragraphs. Each element is one paragraph/heading/list. */
export function textToParagraphs(raw: string): string[] {
  const paragraphs = raw.split(/\n\n+/);
  const result: string[] = [];
  let inList = false;
  let listItems: string[] = [];

  const flushList = () => {
    if (listItems.length > 0) {
      result.push("<ul>\n" + listItems.join("\n") + "\n</ul>");
      listItems = [];
    }
    inList = false;
  };

  for (const p of paragraphs) {
    const trimmed = p.trim();
    if (!trimmed) continue;

    // Heading marker from PDF extractor
    if (trimmed.startsWith("__H2__")) {
      flushList();
      result.push(`<h2>${escapeHtml(trimmed.slice(6))}</h2>`);
      continue;
    }

    // Heading detected by text pattern (fallback for pasted text)
    if (isHeading(trimmed)) {
      flushList();
      result.push(`<h2>${escapeHtml(trimmed)}</h2>`);
      continue;
    }

    // List items
    const listLines = trimmed.split("\n").map((s) => s.trim()).filter(Boolean);
    if (listLines.every(isListItem)) {
      inList = true;
      for (const l of listLines) {
        listItems.push(`<li>${escapeHtml(l.replace(/^[-*•]\s/, "").replace(/^\d+[.)]\s/, ""))}</li>`);
      }
      continue;
    }

    // Regular paragraph
    flushList();
    result.push(`<p>${escapeHtml(trimmed)}</p>`);
  }

  flushList();
  return result;
}

/** Convert paragraphs (separated by double newlines) to HTML. Handles `__H2__` heading markers. */
export function textToHtml(raw: string): string {
  return textToParagraphs(raw).join("\n");
}

/** Segment text into larger blocks (for immersive reading). Groups HTML paragraphs into blocks of ~800 words. */
export function segmentTextToBlocks(html: string, wordsPerBlock: number = 800): string[] {
  // Split on paragraph boundaries
  const paragraphs = html.split(/<\/p>\s*<p>/);
  const blocks: string[] = [];
  let currentBlock = "";
  let currentWordCount = 0;

  for (let i = 0; i < paragraphs.length; i++) {
    let p = paragraphs[i]!;
    // Clean up tags
    p = p.replace(/^<p>/, "").replace(/<\/p>$/, "");
    const wordCount = p.split(/\s+/).filter(Boolean).length;

    if (currentBlock && currentWordCount + wordCount > wordsPerBlock) {
      blocks.push(currentBlock);
      currentBlock = `<p>${p}</p>`;
      currentWordCount = wordCount;
    } else {
      currentBlock += (currentBlock ? "\n" : "") + `<p>${p}</p>`;
      currentWordCount += wordCount;
    }
  }
  if (currentBlock) blocks.push(currentBlock);

  return blocks.length > 0 ? blocks : [html];
}

// Keep old functions for backward compatibility

/** Segment raw text into blocks. Splits on double newlines, falling back to single newlines. */
export function segmentText(raw: string): string[] {
  const cleaned = raw.replace(/\r\n/g, "\n").trim();
  if (!cleaned) return [];

  // Primary split on double newlines (paragraph / block boundaries)
  let parts: string[] = [];
  if (cleaned.includes("\n\n")) {
    parts = cleaned
      .split("\n\n")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }

  // Fallback: if there are no double newlines, split on single newlines but
  // merge lines into reasonable paragraphs (every ~4 lines or on sentence end).
  if (parts.length <= 1) {
    const lines = cleaned
      .split("\n")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    const merged: string[] = [];
    let current = "";
    for (const line of lines) {
      if (current && (line.length < 40 || current.length > 600)) {
        merged.push(current);
        current = line;
      } else {
        current = current ? `${current} ${line}` : line;
      }
    }
    if (current) merged.push(current);
    parts = merged;
  }

  return parts.filter((s) => s.length > 0);
}

/** Filter repeated header/footer lines that appear in most blocks (e.g. page numbers, journal names). */
export function filterRepeatedLines(blocks: string[]): string[] {
  if (blocks.length < 4) return blocks;

  const counts = new Map<string, number>();
  for (const b of blocks) {
    const lines = b.split("\n").map((s) => s.trim()).filter((s) => s.length > 0);
    for (const line of lines) {
      if (line.length < 60) {
        counts.set(line, (counts.get(line) ?? 0) + 1);
      }
    }
  }

  const repeated = new Set<string>();
  for (const [line, count] of counts) {
    if (count >= Math.max(2, blocks.length * 0.3)) {
      repeated.add(line);
    }
  }

  if (repeated.size === 0) return blocks;

  return blocks
    .map((b) => {
      const lines = b.split("\n");
      const kept = lines.filter((l) => !repeated.has(l.trim()) || l.trim().length === 0);
      return kept.join("\n").trim();
    })
    .filter((s) => s.length > 0);
}

/** Clean hyphenation: "assess-\nment" → "assessment" (but keep "-\n" as en-dash). */
export function cleanHyphenation(text: string): string {
  return text.replace(/([a-z])-\n([a-z])/gi, "$1$2");
}

/** Extract plain text from a pasted/dropped string, cleaning up common OCR artifacts. */
export function cleanExtractedText(raw: string): string {
  let t = raw.replace(/\r\n/g, "\n");
  t = t.replace(/[ \t]+/g, " ");
  t = t.replace(/\n{3,}/g, "\n\n");
  return t.trim();
}