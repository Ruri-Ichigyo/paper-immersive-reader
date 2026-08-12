import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  FormControl,
  InputLabel,
  MenuItem,
  Paper as MuiPaper,
  Select,
  TextField,
  Typography,
} from "@mui/material";
import { MenuBook } from "@mui/icons-material";
import * as db from "../db";
import type { Annotation, Paper } from "../types";

export default function AnnotationsPage() {
  const navigate = useNavigate();
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [papers, setPapers] = useState<Record<string, Paper>>({});
  const [search, setSearch] = useState("");
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [tagFilter, setTagFilter] = useState<string>("all");

  const load = useCallback(async () => {
    const [anns, storedPapers] = await Promise.all([db.loadAllAnnotations(), db.listPapers()]);
    setAnnotations(anns);
    const pmap: Record<string, Paper> = {};
    for (const sp of storedPapers) pmap[sp.paper.id[0]] = sp.paper;
    setPapers(pmap);
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const allTags = Array.from(new Set(annotations.flatMap((a) => a.tags)));

  const filtered = annotations.filter((a) => {
    const matchSearch =
      !search ||
      a.quote.toLowerCase().includes(search.toLowerCase()) ||
      (a.note ?? "").toLowerCase().includes(search.toLowerCase());
    const matchType = typeFilter === "all" || a.annotation_type === typeFilter;
    const matchTag = tagFilter === "all" || a.tags.includes(tagFilter);
    return matchSearch && matchType && matchTag;
  });

  const typeColor = (t: string) =>
    t === "Highlight" ? "warning" : t === "Underline" ? "info" : "success";

  return (
    <Box>
      <Typography variant="h5" sx={{ mb: 2 }}>
        All Annotations ({annotations.length})
      </Typography>

      <Box sx={{ display: "flex", gap: 2, mb: 2, flexWrap: "wrap" }}>
        <TextField
          label="Search text or note"
          size="small"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          sx={{ flex: 1, minWidth: 200 }}
        />
        <FormControl size="small" sx={{ width: 140 }}>
          <InputLabel>Type</InputLabel>
          <Select label="Type" value={typeFilter} onChange={(e) => setTypeFilter(e.target.value)}>
            <MenuItem value="all">All</MenuItem>
            <MenuItem value="Highlight">Highlight</MenuItem>
            <MenuItem value="Underline">Underline</MenuItem>
            <MenuItem value="Note">Note</MenuItem>
          </Select>
        </FormControl>
        <FormControl size="small" sx={{ width: 140 }}>
          <InputLabel>Tag</InputLabel>
          <Select label="Tag" value={tagFilter} onChange={(e) => setTagFilter(e.target.value)}>
            <MenuItem value="all">All</MenuItem>
            {allTags.map((t) => (
              <MenuItem key={t} value={t}>{t}</MenuItem>
            ))}
          </Select>
        </FormControl>
      </Box>

      {filtered.length === 0 ? (
        <MuiPaper sx={{ p: 4, textAlign: "center" }}>
          <Typography color="text.secondary">
            {annotations.length === 0
              ? "No annotations yet. Create one while reading."
              : "No annotations match your filters."}
          </Typography>
        </MuiPaper>
      ) : (
        filtered.map((a) => {
          const paper = papers[a.paper_id[0]];
          return (
            <Card key={a.id} variant="outlined" sx={{ mb: 1.5 }}>
              <CardContent>
                <Box sx={{ display: "flex", gap: 0.5, mb: 0.5, flexWrap: "wrap", alignItems: "center" }}>
                  <Chip size="small" label={a.annotation_type} color={typeColor(a.annotation_type) as "warning"} variant="outlined" />
                  {a.tags.map((t) => (
                    <Chip key={t} size="small" label={t} />
                  ))}
                </Box>
                <Typography variant="body2" sx={{ fontStyle: "italic" }} color="text.secondary">
                  “{a.quote}”
                </Typography>
                {a.note && <Typography variant="body2" sx={{ mt: 0.5 }}>{a.note}</Typography>}
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mt: 1 }}>
                  <Typography variant="caption" color="text.secondary">
                    {paper?.title ?? "Unknown paper"}
                  </Typography>
                  <Button
                    size="small"
                    startIcon={<MenuBook />}
                    onClick={() => navigate(`/read/${a.paper_id[0]}`)}
                  >
                    Open
                  </Button>
                </Box>
              </CardContent>
            </Card>
          );
        })
      )}
    </Box>
  );
}