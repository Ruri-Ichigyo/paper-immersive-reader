import { useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Box,
  Button,
  Card,
  CardContent,
  Grid,
  Paper as MuiPaper,
  Stepper,
  Step,
  StepLabel,
  TextField,
  Typography,
  Alert,
  CircularProgress,
} from "@mui/material";
import { UploadFile, ContentPaste } from "@mui/icons-material";
import { useApp } from "../context/AppContext";
import { extractPdfText } from "../utils/pdf";
import { cleanExtractedText, textToParagraphs } from "../utils/text";
import type { Block, Paper } from "../types";

function shortId(prefix: string): string {
  const rand = Math.random().toString(36).slice(2, 8);
  return `${prefix}-${Date.now().toString(36)}-${rand}`;
}

type Stage = "idle" | "extract" | "clean" | "segment" | "save" | "done";

export default function ImportPage() {
  const { savePaper, saveBlocks } = useApp();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<"pdf" | "paste">("pdf");
  const [stage, setStage] = useState<Stage>("idle");
  const [error, setError] = useState<string | null>(null);
  const [pasteText, setPasteText] = useState("");
  const [pdfFile, setPdfFile] = useState<File | null>(null);
  const [title, setTitle] = useState("");

  const runPdfImport = async (file: File) => {
    setError(null);
    setStage("extract");
    try {
      const raw = await extractPdfText(file);
      setStage("segment");
      if (!raw.trim()) {
        setError("No readable text found in the PDF. It may be a scanned document.");
        setStage("idle");
        return;
      }
      const paragraphs = textToParagraphs(raw);
      await commitPaper(paragraphs, file.name.replace(/\.pdf$/i, ""), "Pdf");
    } catch (e) {
      setError(`Failed to import PDF: ${e instanceof Error ? e.message : String(e)}`);
      setStage("idle");
    }
  };

  const runPasteImport = async () => {
    const cleaned = cleanExtractedText(pasteText);
    if (!cleaned.trim()) {
      setError("Please paste some text to import.");
      return;
    }
    setError(null);
    setStage("segment");
    const paragraphs = textToParagraphs(cleaned);
    // Strip HTML tags for the title preview
    const plainPreview = paragraphs[0]?.replace(/<[^>]*>/g, "").slice(0, 60) || "";
    const pasteTitle = title.trim() || plainPreview || "Pasted text";
    await commitPaper(paragraphs, pasteTitle, "Paste");
  };

  const commitPaper = async (blockTexts: string[], paperTitle: string, sourceType: Paper["source_type"]) => {
    setStage("save");
    const now = Date.now();
    const paperId = shortId("paper");
    const paper: Paper = {
      id: { 0: paperId },
      title: paperTitle,
      source_type: sourceType,
      source_ref: null,
      status: "ReadyText",
      parse_version: 1,
    };
    await savePaper(paper, now, now);
    const blocks: Block[] = blockTexts.map((text, i) => ({
      id: { 0: shortId("block") },
      paper_id: { 0: paperId },
      order_index: i,
      text,
      block_hash: null,
    }));
    await saveBlocks(blocks);
    setStage("done");
    setTimeout(() => navigate(`/read/${paperId}`), 400);
  };

  return (
    <Box>
      <Typography variant="h5" sx={{ mb: 3 }}>
        Import Paper
      </Typography>

      <Grid container spacing={2}>
        <Grid size={{ xs: 12, md: 6 }}>
          <MuiPaper
            variant={activeTab === "pdf" ? "outlined" : undefined}
            sx={{
              p: 3,
              cursor: "pointer",
              border: activeTab === "pdf" ? 2 : 1,
              borderColor: activeTab === "pdf" ? "primary.main" : "divider",
            }}
            onClick={() => setActiveTab("pdf")}
          >
            <Typography variant="h6" gutterBottom>
              <UploadFile sx={{ verticalAlign: "middle", mr: 1 }} />
              Import PDF
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              Upload a local PDF file. Text is extracted using pdf.js.
            </Typography>
            <input
              type="file"
              accept="application/pdf"
              style={{ display: "none" }}
              id="pdf-upload-input"
              onChange={(e) => {
                const f = e.target.files?.[0] ?? null;
                setPdfFile(f);
                if (f) {
                  setActiveTab("pdf");
                  runPdfImport(f);
                }
              }}
            />
            <label htmlFor="pdf-upload-input">
              <Button variant="contained" component="span" startIcon={<UploadFile />}>
                Choose PDF file
              </Button>
            </label>
            {pdfFile && (
              <Typography variant="caption" sx={{ display: "block", mt: 1, color: "text.secondary" }}>
                {pdfFile.name}
              </Typography>
            )}
          </MuiPaper>
        </Grid>

        <Grid size={{ xs: 12, md: 6 }}>
          <MuiPaper
            variant={activeTab === "paste" ? "outlined" : undefined}
            sx={{
              p: 3,
              cursor: "pointer",
              border: activeTab === "paste" ? 2 : 1,
              borderColor: activeTab === "paste" ? "primary.main" : "divider",
            }}
            onClick={() => setActiveTab("paste")}
          >
            <Typography variant="h6" gutterBottom>
              <ContentPaste sx={{ verticalAlign: "middle", mr: 1 }} />
              Paste Text
            </Typography>
            {activeTab === "paste" && (
              <Box>
                <TextField
                  label="Title (optional)"
                  fullWidth
                  size="small"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  sx={{ mb: 1.5 }}
                />
                <TextField
                  label="Paste article text here"
                  multiline
                  minRows={8}
                  fullWidth
                  value={pasteText}
                  onChange={(e) => setPasteText(e.target.value)}
                  sx={{ mb: 2 }}
                />
                <Button
                  variant="contained"
                  startIcon={<ContentPaste />}
                  onClick={runPasteImport}
                  disabled={!pasteText.trim()}
                >
                  Import Text
                </Button>
              </Box>
            )}
            {activeTab !== "paste" && (
              <Typography variant="body2" color="text.secondary">
                Click to paste article text directly.
              </Typography>
            )}
          </MuiPaper>
        </Grid>
      </Grid>

      {stage !== "idle" && stage !== "done" && (
        <Card sx={{ mt: 3 }}>
          <CardContent>
            <Stepper activeStep={stage === "extract" ? 0 : stage === "clean" ? 1 : stage === "segment" ? 2 : 3}>
              <Step><StepLabel>Extracting text</StepLabel></Step>
              <Step><StepLabel>Cleaning headers</StepLabel></Step>
              <Step><StepLabel>Segmenting blocks</StepLabel></Step>
              <Step><StepLabel>Saving</StepLabel></Step>
            </Stepper>
            <Box sx={{ display: "flex", alignItems: "center", gap: 2, mt: 2 }}>
              <CircularProgress size={20} />
              <Typography color="text.secondary">
                {stage === "extract" && "Extracting text from PDF…"}
                {stage === "segment" && "Segmenting text into reading blocks…"}
                {stage === "save" && "Saving to local database…"}
              </Typography>
            </Box>
          </CardContent>
        </Card>
      )}

      {stage === "done" && (
        <Alert severity="success" sx={{ mt: 3 }}>
          Import successful! Opening the reader…
        </Alert>
      )}

      {error && (
        <Alert severity="error" sx={{ mt: 3 }}>
          {error}
        </Alert>
      )}
    </Box>
  );
}