# PRD — English Paper Immersive Reader (Web-first, Local-first)

## 0. Document Control
- Product name (working): **Paper Immersive Reader**
- Target platform: **Web first**, reserved extension to **PC / App** later
- Design principles (must):
  - **Local-first** (offline usable, user owns data)
  - **Readability-first** AI assistance (may add background knowledge moderately, but must not deviate from paper semantics)
  - **No sharing / collaboration features**
  - User-managed **data storage & sync**

## 1. Product Overview

### 1.1 Positioning
A reading tool for **graduate students and researchers** to read English academic papers with:
- immersive block-by-block (sentence/chunk) reading
- word/selection translation
- AI terminology explanations and comprehension assistance
- annotations (highlight/underline/note/tags)
- vocabulary list (auto-collected from lookups)
- reading progress + time statistics
- optional export of learning outcomes

### 1.2 Target Users
Primary:
- graduate students
- researchers
- anyone reading English papers at high frequency

### 1.3 User Goals
- understand paper faster
- reduce friction for long sentences and domain terms
- record highlights/questions/vocabulary during reading
- keep progress and accumulate knowledge across papers

## 2. Scope & Non-goals

### 2.1 In-scope (MVP / P0 closed loop)
Import paper → parse → immersive reading → translate selection → AI term explanation → annotate → progress saved → vocabulary auto-collected.

### 2.2 Explicit Non-goals (hard constraints)
- **No sharing features** (no public links, team spaces, collaborative annotation)
- Not “cloud-first” centralized storage (sync is user-managed)
- No manual control for AI chunking intensity (chunk length is AI-adaptive)

### 2.3 Deferred / P1-P2 (allowed later)
- AI summarization (paragraph / full paper)
- structure extraction (purpose/method/results/conclusion)
- Q&A
- note generation
- keyword extraction, key sentence marking
- annotation search + export by category
- OCR for scanned PDFs
- Zotero bi-directional sync
- S3-compatible sync
- PC/App expansion

## 3. Core Concepts & Definitions

### 3.1 Paper (Reading Object)
A single imported literature item with:
- metadata (title, source, import method, timestamps)
- original asset (PDF or plain text)
- parsed content (paragraphs/blocks/sentences)
- reading state (position/progress)
- user data (annotations, vocabulary, AI records)

### 3.2 Block / Chunk
A unit presented in immersive reading mode.
- Created by **AI adaptive chunking** (FR-12)
- Can represent 1 sentence, multiple sentences, or a short paragraph segment

### 3.3 Annotation
User-created highlight/underline/note/tag anchored to text ranges.
- Default tags: **重点 / 疑问 / 术语 / 结论 / 复习**
- User can create additional tags

### 3.4 Vocabulary Item
A word/phrase the user looked up via selection translation.
- Auto-collected into vocabulary list

## 4. Functional Requirements (FR)

> Note: IDs keep the user-provided numbering.

### 4.1 Import
- **FR-01** Support import sources:
  1) local PDF file
  2) paste text
  3) web URL
  4) DOI / paper link
  5) Zotero import
- **FR-02** After import, create a unique paper object and persist metadata (title/source/import time/progress etc.)

### 4.2 PDF Parsing & Pre-processing
- **FR-03** Extract main text from PDF; filter non-body content as much as possible
- **FR-04** Auto-paragraph segmentation
- **FR-05** Remove header/footer repeated noise
- **FR-06** OCR for scanned/image PDFs

### 4.3 Reading Mode
- **FR-07** Dedicated reading page (distraction-free)
- **FR-08** Fullscreen immersive reading; hide irrelevant UI
- **FR-09** Show reading progress
- **FR-10** Sentence/block reading from AI segmentation
- **FR-11** “Next block” continues reading
- **FR-12** AI adaptive chunking; user does not configure intensity

### 4.4 Translation
- **FR-13** Selection translation (word/sentence)
- **FR-14** Paragraph translation
- **FR-15** Bilingual display: English left, Chinese right

### 4.5 AI Explanation & Assistance
- **FR-16** Term explanation in right AI panel
- **FR-17** Explanation language is switchable via settings
- **FR-18** Sentence explanation
- **FR-19** Paragraph/full summary
- **FR-20** Structure extraction
- **FR-21** Q&A
- **FR-22** Generate notes
- **FR-23** Analyze topic/logic/core viewpoints

### 4.6 Annotations
- **FR-24** Sentence-level annotations
- **FR-25** Types: highlight / underline / note / tag
- **FR-26** Default + custom tags
- **FR-27** Search annotations
- **FR-28** Export annotations by category

### 4.7 Vocabulary & Learning Records
- **FR-29** Auto-record looked-up words
- **FR-30** Vocabulary management UI
- **FR-31** Restore last read position
- **FR-32** Reading time statistics
- **FR-33** Mark key sentences
- **FR-34** Keyword extraction

### 4.8 Export
- **FR-35** Default export includes: original text, annotations, summaries, vocabulary
- **FR-36** Default export excludes: translations, term explanations, Q&A logs

### 4.9 Storage & Sync
- **FR-37** Local-first storage (offline capable)
- **FR-38** User config S3 sync parameters: endpoint/bucket/AK/SK
- **FR-39** Sync ALL content: progress/annotations/favorites/vocabulary/AI chat logs
- **FR-40** Upload WITHOUT additional encryption

### 4.10 Zotero
- **FR-41** Zotero bi-directional sync
- **FR-42** Sync granularity: metadata + PDF + annotations/notes

## 5. Data, Privacy, and AI Constraints

### 5.1 Local Data Ownership (non-negotiable)
- All user data must be stored locally by default and usable offline.
- Sync is opt-in and user-configured.

### 5.2 AI Behavior Guardrails
- AI outputs must prioritize **readability** but should not distort paper meaning.
- Where AI adds background knowledge, it must clearly label it as “background” and keep it short.

### 5.3 Sync Security Reality (explicit)
Requirement states user provides AK/SK. For a pure browser-only web app, storing AK/SK in the browser is risky and S3 CORS/credential exposure is a practical issue.

**Product decision (to remove ambiguity):**
- Provide a **user-managed Sync Gateway** (self-hostable) for S3 + Zotero credentials and network operations.
- The web app remains local-first and communicates with Sync Gateway over HTTPS.
- If user refuses gateway, the app may still run fully local without sync.

### 5.4 “No Encryption” Clarification
- “No extra encryption before upload” means: the product will not apply additional client-side encryption.
- Transport security (HTTPS/TLS) is still required.

## 6. Success Criteria (MVP)

MVP is considered successful when:
- User can import a PDF and see clean-ish body text blocks.
- Immersive reading works with reliable “next block” navigation.
- Selection translation works for words and sentences.
- AI panel provides term explanation and sentence explanation.
- Annotations persist across reload.
- Progress and last position restore correctly.
- Vocabulary list auto-collects looked-up items.

## 7. Risks (from requirements)
- OCR accuracy for scanned PDFs may be insufficient.
- PDF parsing quality varies by publisher layout.
- AI explanation hallucination risk, especially for conclusions/terms.
- Sync consistency & conflict handling across S3 + Zotero bi-directional sync.
