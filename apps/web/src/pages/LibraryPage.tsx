import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Box,
  Button,
  Card,
  CardContent,
  CardActions,
  Chip,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Grid,
  LinearProgress,
  Typography,
  IconButton,
  Tooltip,
} from "@mui/material";
import { Add, Delete, Download, PlayArrow } from "@mui/icons-material";
import { useApp } from "../context/AppContext";
import type { Paper, ReadingStats } from "../types";

function formatDuration(ms: number): string {
  const mins = Math.floor(ms / 60000);
  const secs = Math.floor((ms % 60000) / 1000);
  if (mins === 0) return `${secs}s`;
  return `${mins}m ${secs}s`;
}

interface PaperMeta {
  totalBlocks: number;
  currentIndex: number;
  stats?: ReadingStats;
}

export default function LibraryPage() {
  const { papers, refresh, getBlocks, getProgress, getReadingStats, deletePaper } = useApp();
  const navigate = useNavigate();
  const [meta, setMeta] = useState<Record<string, PaperMeta>>({});
  const [deleteTarget, setDeleteTarget] = useState<Paper | null>(null);
  const [exporting, setExporting] = useState<string | null>(null);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    (async () => {
      const m: Record<string, PaperMeta> = {};
      for (const p of papers) {
        const blocks = await getBlocks(p.id[0]);
        const progress = await getProgress(p.id[0]);
        let stats: ReadingStats | undefined;
        try {
          stats = (await getReadingStats(p.id[0])) as ReadingStats | undefined;
        } catch {
          stats = undefined;
        }
        m[p.id[0]] = {
          totalBlocks: blocks.length,
          currentIndex: progress?.current_block_index ?? 0,
          stats,
        };
      }
      setMeta(m);
    })();
  }, [papers, getBlocks, getProgress, getReadingStats]);

  const handleExport = async (paper: Paper) => {
    setExporting(paper.id[0]);
    try {
      const blocks = await getBlocks(paper.id[0]);
      const annotations = await getAnnotations(paper.id[0]);
      const progress = await getProgress(paper.id[0]);
      const stats = await getReadingStats(paper.id[0]);
      const bundle = {
        exported_at: new Date().toISOString(),
        paper,
        blocks,
        annotations,
        progress,
        reading_stats: stats ? { ...stats } : null,
      };
      const blob = new Blob([JSON.stringify(bundle, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${(paper.title ?? paper.id[0]).replace(/[^\w.-]+/g, "_")}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } finally {
      setExporting(null);
    }
  };

  const getAnnotations = useApp().getAnnotations;

  return (
    <Box>
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3 }}>
        <Typography variant="h5">Library</Typography>
        <Button variant="contained" startIcon={<Add />} onClick={() => navigate("/import")}>
          Import Paper
        </Button>
      </Box>

      {papers.length === 0 ? (
        <Card>
          <CardContent sx={{ textAlign: "center", py: 6 }}>
            <Typography variant="h6" color="text.secondary" gutterBottom>
              No papers yet
            </Typography>
            <Typography color="text.secondary" sx={{ mb: 2 }}>
              Import a PDF or paste text to start reading.
            </Typography>
            <Button variant="contained" startIcon={<Add />} onClick={() => navigate("/import")}>
              Import Paper
            </Button>
          </CardContent>
        </Card>
      ) : (
        <Grid container spacing={2}>
          {papers.map((p) => {
            const m = meta[p.id[0]];
            const pct = m && m.totalBlocks > 0
              ? Math.min(100, Math.round(((m.currentIndex + 1) / m.totalBlocks) * 100))
              : 0;
            return (
              <Grid size={{ xs: 12, sm: 6, lg: 4 }} key={p.id[0]}>
                <Card variant="outlined">
                  <CardContent>
                    <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", mb: 1 }}>
                      <Typography variant="subtitle1" sx={{ fontWeight: 600, mr: 1 }}>
                        {p.title ?? "Untitled paper"}
                      </Typography>
                      <Chip size="small" color="primary" variant="outlined" label={p.source_type} />
                    </Box>
                    <Typography variant="caption" color="text.secondary">
                      {p.id[0]}
                    </Typography>
                    {m && m.totalBlocks > 0 && (
                      <Box sx={{ mt: 2 }}>
                        <Typography variant="caption" color="text.secondary">
                          Block {Math.min(m.currentIndex + 1, m.totalBlocks)} of {m.totalBlocks} · {pct}%
                        </Typography>
                        <LinearProgress variant="determinate" value={pct} sx={{ mt: 0.5, height: 6, borderRadius: 3 }} />
                      </Box>
                    )}
                    {m?.stats && m.stats.total_reading_ms > 0 && (
                      <Typography variant="caption" color="text.secondary" sx={{ display: "block", mt: 1 }}>
                        Read {formatDuration(m.stats.total_reading_ms)} · {m.stats.session_count} session
                        {m.stats.session_count === 1 ? "" : "s"}
                      </Typography>
                    )}
                  </CardContent>
                  <CardActions>
                    <Button size="small" startIcon={<PlayArrow />} onClick={() => navigate(`/read/${p.id[0]}`)}>
                      Read
                    </Button>
                    <Box sx={{ flexGrow: 1 }} />
                    <Tooltip title="Export JSON">
                      <IconButton size="small" onClick={() => handleExport(p)} disabled={exporting === p.id[0]}>
                        {exporting === p.id[0] ? <CircularProgress size={18} /> : <Download />}
                      </IconButton>
                    </Tooltip>
                    <Tooltip title="Delete">
                      <IconButton size="small" color="error" onClick={() => setDeleteTarget(p)}>
                        <Delete />
                      </IconButton>
                    </Tooltip>
                  </CardActions>
                </Card>
              </Grid>
            );
          })}
        </Grid>
      )}

      <Dialog open={!!deleteTarget} onClose={() => setDeleteTarget(null)}>
        <DialogTitle>Delete Paper?</DialogTitle>
        <DialogContent>
          <DialogContentText>
            This will permanently delete "{deleteTarget?.title ?? "this paper"}" and all its blocks,
            annotations, vocabulary, and reading history.
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteTarget(null)}>Cancel</Button>
          <Button
            color="error"
            variant="contained"
            onClick={async () => {
              if (deleteTarget) await deletePaper(deleteTarget.id[0]);
              setDeleteTarget(null);
            }}
          >
            Delete
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}