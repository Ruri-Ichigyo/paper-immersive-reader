import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  FormControl,
  IconButton,
  InputLabel,
  LinearProgress,
  MenuItem,
  Paper,
  Select,
  TextField,
  Tooltip,
  Typography,
} from "@mui/material";
import {
  ArrowBack,
  ArrowForward,
  Fullscreen,
  FullscreenExit,
  MenuOpen,
  Translate,
} from "@mui/icons-material";
import { useApp } from "../context/AppContext";
import TextSelectionPopover, { type SelectionActionKind } from "../components/TextSelectionPopover";
import type { Annotation, AnnotationType, Block } from "../types";
import { DEFAULT_TAGS } from "../types";

interface SelectionState {
  text: string;
  start: number;
  end: number;
}

export default function ReadPage() {
  const { paperId } = useParams();
  const navigate = useNavigate();
  const app = useApp();

  const [paper, setPaper] = useState<{ title: string | null } | null>(null);
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [selection, setSelection] = useState<SelectionState | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [fullscreen, setFullscreen] = useState(false);

  // Translation states
  const [transSource, setTransSource] = useState<string | null>(null);
  const [transResult, setTransResult] = useState<string | null>(null);
  const [transLoading, setTransLoading] = useState(false);
  const [transError, setTransError] = useState<string | null>(null);

  // AI explanation states
  const [aiContent, setAiContent] = useState<string | null>(null);
  const [aiLoading, setAiLoading] = useState(false);
  const [aiType, setAiType] = useState("");

  // Annotation form
  const [noteOpen, setNoteOpen] = useState(false);
  const [noteType, setNoteType] = useState<AnnotationType>("Highlight");
  const [noteText, setNoteText] = useState("");
  const [noteTag, setNoteTag] = useState("");

  const blockRef = useRef<HTMLDivElement>(null);
  const sessionStartRef = useRef<number>(Date.now());

  const load = useCallback(async () => {
    if (!paperId) return;
    const paperData = await app.getPaper(paperId);
    if (paperData) setPaper({ title: paperData.title });
    const blocksData = await app.getBlocks(paperId);
    setBlocks(blocksData);
    const progress = await app.getProgress(paperId);
    const idx = progress
      ? Math.min(progress.current_block_index, Math.max(0, blocksData.length - 1))
      : 0;
    setCurrentIndex(idx);
    const anns = await app.getAnnotations(paperId);
    setAnnotations(anns);
    try {
      const meta = await app.getReadingStats(paperId);
      if (meta) {
        sessionStartRef.current = Date.now();
        await app.saveReadingStats({ ...meta, session_started_ms: Date.now() });
      } else {
        await app.saveReadingStats({
          paper_id: { 0: paperId },
          total_reading_ms: 0,
          session_started_ms: Date.now(),
          session_count: 0,
        });
      }
    } catch {
      // ignore stats errors
    }
  }, [paperId, app]);

  useEffect(() => {
    load();
  }, [load]);

  // Persist progress whenever index changes
  const persistCurrent = useCallback(
    async (idx: number) => {
      if (!paperId) return;
      const block = blocks[idx];
      await app.saveProgress({
        paper_id: { 0: paperId },
        current_block_index: idx,
        current_block_id: block ? block.id : null,
        updated_at_ms: Date.now(),
      });
    },
    [paperId, blocks, app],
  );

  useEffect(() => {
    if (blocks.length > 0) persistCurrent(currentIndex);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentIndex]);

  // Persist reading time on unmount
  useEffect(() => {
    return () => {
      if (!paperId) return;
      const elapsed = Date.now() - sessionStartRef.current;
      if (elapsed >= 1000) {
        app.getReadingStats(paperId).then((stats) => {
          const base = stats ?? {
            paper_id: { 0: paperId },
            total_reading_ms: 0,
            session_started_ms: null,
            session_count: 0,
          };
          const updated = {
            ...base,
            total_reading_ms: base.total_reading_ms + elapsed,
            session_count: base.session_count + 1,
            session_started_ms: null,
          };
          app.saveReadingStats(updated);
        });
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [paperId]);

  // Keyboard navigation
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.key === "ArrowRight" || e.key === "PageDown" || e.key === " ") {
        e.preventDefault();
        setCurrentIndex((i) => Math.min(i + 1, blocks.length - 1));
      } else if (e.key === "ArrowLeft" || e.key === "PageUp") {
        e.preventDefault();
        setCurrentIndex((i) => Math.max(i - 1, 0));
      } else if (e.key === "f" || e.key === "F") {
        setFullscreen((f) => !f);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [blocks.length]);

  // Text selection handling
  const handleMouseUp = useCallback(() => {
    const sel = window.getSelection();
    const text = sel?.toString().trim() ?? "";
    if (!text) {
      setSelection(null);
      return;
    }
    // Use requestAnimationFrame to ensure selection is finalised
    requestAnimationFrame(() => {
      const s = window.getSelection();
      const t = s?.toString().trim() ?? "";
      if (!t) {
        setSelection(null);
        return;
      }
      const blockText = blocks[currentIndex]?.text ?? "";
      const start = blockText.indexOf(t);
      if (start >= 0) {
        setSelection({ text: t, start, end: start + t.length });
      } else {
        setSelection({ text: t, start: 0, end: 0 });
      }
    });
  }, [blocks, currentIndex]);

  const handleSelectionAction = useCallback(
    async (kind: SelectionActionKind) => {
      if (!paperId || !selection) return;
      const selText = selection.text;
      const block = blocks[currentIndex];
      const blockId = block ? block.id : { 0: "" };

      if (kind === "note") {
        setNoteOpen(true);
        setSelection(null);
        return;
      }

      if (kind === "translate") {
        setTransSource(selText);
        setTransResult(null);
        setTransError(null);
        setTransLoading(true);
        // auto-collect vocabulary for single words
        if (selText.split(/\s+/).length === 1) {
          await app.upsertVocabulary({
            id: `vocab-${Date.now().toString(36)}`,
            term: selText,
            term_lower: selText.toLowerCase(),
            paper_id_first_seen: { 0: paperId },
            block_id_first_seen: blockId,
            lookup_count: 1,
            first_seen_at_ms: Date.now(),
            last_seen_at_ms: Date.now(),
            note: null,
            learned: false,
          });
        }
        try {
          const lang = app.settings.explanation_language === "zh" ? "zh" : "zh";
          const result = await app.gatewayClient.translate(selText, lang);
          setTransResult(result);
          const now = Date.now();
          await app.saveAiRecord({
            id: `ai-${now.toString(36)}`,
            paper_id: { 0: paperId },
            block_id: blockId,
            record_type: "translate",
            input: selText,
            output: result,
            language: lang,
            created_at_ms: now,
          });
        } catch (e) {
          setTransError(e instanceof Error ? e.message : String(e));
        } finally {
          setTransLoading(false);
        }
      }

      if (kind === "term" || kind === "sentence") {
        const expType = kind === "term" ? "term" : "sentence";
        setAiType(expType === "term" ? "Term Explanation" : "Sentence Explanation");
        setAiContent(null);
        setAiLoading(true);
        try {
          const lang = app.settings.explanation_language;
          const result = await app.gatewayClient.explain(selText, expType, lang, block.text);
          setAiContent(result.output);
          const now = Date.now();
          await app.saveAiRecord({
            id: `ai-${now.toString(36)}`,
            paper_id: { 0: paperId },
            block_id: blockId,
            record_type: `explain_${expType}`,
            input: selText,
            output: result.output,
            language: lang,
            created_at_ms: now,
          });
        } catch (e) {
          setAiContent(`Error: ${e instanceof Error ? e.message : String(e)}`);
        } finally {
          setAiLoading(false);
        }
      }

      setSelection(null);
    },
    [paperId, selection, blocks, currentIndex, app],
  );

  const translateBlock = useCallback(async () => {
    if (!paperId) return;
    const block = blocks[currentIndex];
    if (!block) return;
    setTransSource(block.text);
    setTransResult(null);
    setTransError(null);
    setTransLoading(true);
    try {
      const result = await app.gatewayClient.translate(block.text, "zh");
      setTransResult(result);
      const now = Date.now();
      await app.saveAiRecord({
        id: `ai-${now.toString(36)}`,
        paper_id: { 0: paperId },
        block_id: block.id,
        record_type: "translate_block",
        input: block.text,
        output: result,
        language: "zh",
        created_at_ms: now,
      });
    } catch (e) {
      setTransError(e instanceof Error ? e.message : String(e));
    } finally {
      setTransLoading(false);
    }
  }, [paperId, blocks, currentIndex, app]);

  const submitAnnotation = async () => {
    if (!paperId || !selection) return;
    const block = blocks[currentIndex];
    if (!block) return;
    const ann: Annotation = {
      id: `ann-${Date.now().toString(36)}`,
      paper_id: { 0: paperId },
      block_id: block.id,
      annotation_type: noteType,
      quote: selection.text,
      prefix: null,
      suffix: null,
      start_offset: selection.start,
      end_offset: selection.end,
      note: noteText.trim() ? noteText.trim() : null,
      tags: noteTag ? [noteTag] : [],
      anchoring_status: "Anchored",
    };
    await app.saveAnnotation(ann);
    const anns = await app.getAnnotations(paperId);
    setAnnotations(anns);
    setNoteOpen(false);
    setNoteText("");
    setNoteTag("");
    setSelection(null);
  };

  const deleteAnnotation = async (id: string) => {
    if (!paperId) return;
    await app.deleteAnnotation(id);
    const anns = await app.getAnnotations(paperId);
    setAnnotations(anns);
  };

  const updateAnnotation = async (ann: Annotation) => {
    await app.updateAnnotation(ann);
    if (paperId) {
      const anns = await app.getAnnotations(paperId);
      setAnnotations(anns);
    }
  };

  const currentBlock = blocks[currentIndex];
  const progressPct = blocks.length > 0 ? Math.round(((currentIndex + 1) / blocks.length) * 100) : 0;

  return (
    <Box sx={{ display: "flex", gap: 2, height: "calc(100vh - 120px)" }}>
      <Paper
        elevation={0}
        variant="outlined"
        sx={{
          flex: 1,
          p: 3,
          display: "flex",
          flexDirection: "column",
          overflow: "auto",
          maxWidth: fullscreen ? "100%" : undefined,
        }}
      >
        {/* Toolbar */}
        <Box sx={{ mb: 2 }}>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 1 }}>
            <Tooltip title="Back to Library">
              <IconButton onClick={() => navigate("/library")} size="small">
                <ArrowBack />
              </IconButton>
            </Tooltip>
            <Typography variant="h6" sx={{ flex: 1, mr: 1 }}>
              {paper?.title ?? "Loading…"}
            </Typography>
            <Button
              size="small"
              variant="outlined"
              startIcon={<Translate />}
              onClick={translateBlock}
              disabled={transLoading}
            >
              {transLoading ? "Translating…" : "Translate Block"}
            </Button>
            <Tooltip title={sidebarOpen ? "Hide Panel" : "Show Panel"}>
              <IconButton onClick={() => setSidebarOpen((v) => !v)} size="small">
                <MenuOpen sx={{ transform: sidebarOpen ? "none" : "rotate(180deg)" }} />
              </IconButton>
            </Tooltip>
            <Tooltip title={fullscreen ? "Exit Fullscreen (F)" : "Fullscreen (F)"}>
              <IconButton onClick={() => setFullscreen((v) => !v)} size="small">
                {fullscreen ? <FullscreenExit /> : <Fullscreen />}
              </IconButton>
            </Tooltip>
          </Box>
          <LinearProgress variant="determinate" value={progressPct} sx={{ height: 6, borderRadius: 3 }} />
          <Typography variant="caption" color="text.secondary">
            Paragraph {currentIndex + 1} of {blocks.length} · {progressPct}%
          </Typography>
        </Box>

        {/* Reading content */}
        <Box
          ref={blockRef}
          onMouseUp={handleMouseUp}
          onMouseDown={(e) => {
            // clear selection when clicking the reading area (not popover)
            if (!(e.target as HTMLElement).closest("[data-popover]")) setSelection(null);
          }}
          className="reading-content"
          sx={{
            flex: 1,
            display: "flex",
            flexDirection: "column",
            justifyContent: "center",
            alignItems: "center",
            fontFamily: "'Merriweather', 'Georgia', serif",
            fontSize: "1.15rem",
            lineHeight: 2,
            color: "text.primary",
            userSelect: "text",
            cursor: "text",
            overflowWrap: "break-word",
            wordBreak: "break-word",
            hyphens: "auto",
            maxWidth: "75ch",
            mx: "auto",
            width: "100%",
            "& p": {
              mb: 1.5,
              textIndent: "2em",
              "&:first-of-type": { textIndent: 0 },
            },
            "& h2": {
              fontFamily: "'Inter', sans-serif",
              fontSize: "1.5rem",
              fontWeight: 700,
              mt: 2.5,
              mb: 1.2,
              lineHeight: 1.3,
              textIndent: 0,
              color: "primary.main",
            },
            "& h3": {
              fontFamily: "'Inter', sans-serif",
              fontSize: "1.25rem",
              fontWeight: 600,
              mt: 2,
              mb: 1,
              lineHeight: 1.4,
              textIndent: 0,
            },
            "& ul, & ol": { pl: 2.5, mb: 1.5 },
            "& li": { mb: 0.6, textIndent: 0 },
            "& strong, & b": { fontWeight: 700 },
            "& em, & i": { fontStyle: "italic" },
          }}
        >
          {currentBlock ? (
            <div dangerouslySetInnerHTML={{ __html: currentBlock.text }} />
          ) : (
            <Box sx={{ textAlign: "center", py: 8 }}>
              <Typography color="text.secondary">Paper not found or has no content.</Typography>
              <Button sx={{ mt: 2 }} onClick={() => navigate("/library")} variant="contained">
                Back to Library
              </Button>
            </Box>
          )}
        </Box>

        {/* Navigation */}
        <Box sx={{ display: "flex", justifyContent: "space-between", mt: 2 }}>
          <Button
            variant="outlined"
            startIcon={<ArrowBack />}
            disabled={currentIndex === 0}
            onClick={() => setCurrentIndex((i) => Math.max(i - 1, 0))}
          >
            Previous
          </Button>
          <Button
            variant="contained"
            endIcon={<ArrowForward />}
            disabled={currentIndex >= blocks.length - 1}
            onClick={() => setCurrentIndex((i) => Math.min(i + 1, blocks.length - 1))}
          >
            Next
          </Button>
        </Box>

        {/* Selection popover */}
        <Box data-popover>
          <TextSelectionPopover selection={selection} onAction={handleSelectionAction} onDismiss={() => setSelection(null)} />
        </Box>
      </Paper>

      {/* Sidebar */}
      {sidebarOpen && (
        <Paper elevation={0} variant="outlined" sx={{ width: 360, p: 2, overflow: "auto", flexShrink: 0 }}>
          <Typography variant="subtitle1" sx={{ fontWeight: 600,  mb: 1.5 }}>
            Translator & AI
          </Typography>

          {transLoading && (
            <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 2 }}>
              <CircularProgress size={18} />
              <Typography variant="body2" color="text.secondary">Translating…</Typography>
            </Box>
          )}
          {transSource && transResult && !transLoading && (
            <Card variant="outlined" sx={{ mb: 2 }}>
              <CardContent sx={{ "&:last-child": { pb: 2 } }}>
                <Typography variant="caption" color="text.secondary" sx={{ display: "block" }}>
                  English
                </Typography>
                <Typography variant="body2" sx={{ fontStyle: 'italic',  mb: 1 }}>
                  {transSource}
                </Typography>
                <Divider sx={{ my: 1 }} />
                <Typography variant="caption" color="text.secondary" sx={{ display: "block" }}>
                  {app.settings.explanation_language === "zh" ? "中文" : app.settings.explanation_language}
                </Typography>
                <Typography variant="body2" sx={{ whiteSpace: "pre-wrap" }}>
                  {transResult}
                </Typography>
              </CardContent>
            </Card>
          )}
          {transError && (
            <Typography variant="body2" color="error" sx={{ mb: 2 }}>
              {transError}
            </Typography>
          )}

          {aiLoading && (
            <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 2 }}>
              <CircularProgress size={18} />
              <Typography variant="body2" color="text.secondary">Generating explanation…</Typography>
            </Box>
          )}
          {aiContent && !aiLoading && (
            <Card variant="outlined" sx={{ mb: 2 }}>
              <CardContent sx={{ "&:last-child": { pb: 2 } }}>
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 1 }}>
                  <Typography variant="subtitle2">{aiType}</Typography>
                  <IconButton size="small" onClick={() => setAiContent(null)}>
                    <ArrowForward fontSize="small" sx={{ transform: "rotate(135deg)" }} />
                  </IconButton>
                </Box>
                <Typography variant="body2" sx={{ whiteSpace: "pre-wrap" }}>
                  {aiContent}
                </Typography>
              </CardContent>
            </Card>
          )}

          {!transSource && !aiContent && !aiLoading && !transLoading && (
            <Typography variant="body2" color="text.secondary">
              Select text in the reading area, then choose Translate, Term, or Sentence.
            </Typography>
          )}

          <Divider sx={{ my: 2 }} />
          <Typography variant="subtitle1" sx={{ fontWeight: 600,  mb: 1 }}>
            Annotations ({annotations.length})
          </Typography>
          {annotations.length === 0 ? (
            <Typography variant="body2" color="text.secondary">
              No annotations yet. Select text and choose Note to add one.
            </Typography>
          ) : (
            annotations.map((a) => (
              <AnnotationCard
                key={a.id}
                annotation={a}
                onDelete={() => deleteAnnotation(a.id)}
                onUpdate={updateAnnotation}
              />
            ))
          )}
        </Paper>
      )}

      {/* Annotation dialog */}
      <Dialog open={noteOpen} onClose={() => setNoteOpen(false)}>
        <DialogTitle>Create Annotation</DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ fontStyle: 'italic', mb: 2 }} color="text.secondary">
            “{selection?.text}”
          </Typography>
          <FormControl fullWidth size="small" sx={{ mb: 2 }}>
            <InputLabel>Type</InputLabel>
            <Select
              label="Type"
              value={noteType}
              onChange={(e) => setNoteType(e.target.value as AnnotationType)}
            >
              <MenuItem value="Highlight">Highlight</MenuItem>
              <MenuItem value="Underline">Underline</MenuItem>
              <MenuItem value="Note">Note</MenuItem>
            </Select>
          </FormControl>
          <TextField
            label="Note (optional)"
            fullWidth
            multiline
            minRows={3}
            size="small"
            value={noteText}
            onChange={(e) => setNoteText(e.target.value)}
            sx={{ mb: 2 }}
          />
          <FormControl fullWidth size="small">
            <InputLabel>Tag (optional)</InputLabel>
            <Select label="Tag (optional)" value={noteTag} onChange={(e) => setNoteTag(e.target.value)}>
              <MenuItem value="">- None -</MenuItem>
              {DEFAULT_TAGS.map((t) => (
                <MenuItem key={t} value={t}>{t}</MenuItem>
              ))}
              {app.settings.custom_tags.map((t) => (
                <MenuItem key={t} value={t}>{t}</MenuItem>
              ))}
            </Select>
          </FormControl>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setNoteOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={submitAnnotation}>Save</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

interface AnnotationCardProps {
  annotation: Annotation;
  onDelete: () => void;
  onUpdate: (ann: Annotation) => void;
}

function AnnotationCard({ annotation, onDelete, onUpdate }: AnnotationCardProps) {
  const [editing, setEditing] = useState(false);
  const [note, setNote] = useState(annotation.note ?? "");
  const typeColor =
    annotation.annotation_type === "Highlight"
      ? "warning"
      : annotation.annotation_type === "Underline"
        ? "info"
        : "success";

  return (
    <Card variant="outlined" sx={{ mb: 1, borderLeft: 3, borderLeftColor: "primary.main" }}>
      <CardContent sx={{ "&:last-child": { pb: 1.5 } }}>
        <Chip size="small" label={annotation.annotation_type} color={typeColor as "warning"} variant="outlined" sx={{ mb: 0.5 }} />
        <Typography variant="body2" sx={{ fontStyle: 'italic' }} color="text.secondary">
          “{annotation.quote}”
        </Typography>
        {editing ? (
          <Box sx={{ mt: 1 }}>
            <TextField
              size="small"
              fullWidth
              multiline
              minRows={2}
              value={note}
              onChange={(e) => setNote(e.target.value)}
            />
            <Box sx={{ mt: 1, display: "flex", gap: 1 }}>
              <Button
                size="small"
                variant="contained"
                onClick={() => {
                  onUpdate({ ...annotation, note: note.trim() ? note.trim() : null });
                  setEditing(false);
                }}
              >
                Save
              </Button>
              <Button size="small" onClick={() => { setEditing(false); setNote(annotation.note ?? ""); }}>
                Cancel
              </Button>
            </Box>
          </Box>
        ) : (
          annotation.note && <Typography variant="body2" sx={{ mt: 0.5 }}>{annotation.note}</Typography>
        )}
        {annotation.tags.length > 0 && (
          <Box sx={{ mt: 0.5, display: "flex", gap: 0.5 }}>
            {annotation.tags.map((t) => (
              <Chip key={t} label={t} size="small" variant="outlined" />
            ))}
          </Box>
        )}
        <Box sx={{ mt: 0.5, display: "flex", gap: 0.5 }}>
          <Button size="small" onClick={() => setEditing((v) => !v)}>
            {editing ? "Close" : "Edit"}
          </Button>
          <Button size="small" color="error" onClick={onDelete}>
            Delete
          </Button>
        </Box>
      </CardContent>
    </Card>
  );
}