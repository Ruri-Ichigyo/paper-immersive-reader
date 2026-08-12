import React, { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Box,
  Button,
  Checkbox,
  Chip,
  LinearProgress,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TextField,
  Typography,
  IconButton,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
} from "@mui/material";
import { Delete, Edit, MenuBook } from "@mui/icons-material";
import { useApp } from "../context/AppContext";
import * as db from "../db";
import type { VocabularyItem } from "../types";

export default function VocabularyPage() {
  const { updateVocabulary, deleteVocabulary } = useApp();
  const navigate = useNavigate();
  const [items, setItems] = useState<VocabularyItem[]>([]);
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<"all" | "learning" | "learned">("all");
  const [loading, setLoading] = useState(true);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editNote, setEditNote] = useState("");

  const load = useCallback(async () => {
    const all = await db.listVocabulary();
    setItems(all);
    setLoading(false);
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const filtered = items.filter((it) => {
    const matchSearch = !search || it.term.toLowerCase().includes(search.toLowerCase());
    const matchFilter =
      filter === "all" || (filter === "learned" ? it.learned : !it.learned);
    return matchSearch && matchFilter;
  });

  const learnedCount = items.filter((i) => i.learned).length;
  const pct = items.length > 0 ? Math.round((learnedCount / items.length) * 100) : 0;

  return (
    <Box>
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
        <Typography variant="h5">Vocabulary</Typography>
        <Typography variant="body2" color="text.secondary">
          {learnedCount}/{items.length} learned
        </Typography>
      </Box>

      <LinearProgress variant="determinate" value={pct} sx={{ mb: 3, height: 6, borderRadius: 3 }} />

      <Box sx={{ display: "flex", gap: 2, mb: 2 }}>
        <TextField
          label="Search"
          size="small"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          sx={{ flex: 1 }}
        />
        <FormControl size="small" sx={{ width: 160 }}>
          <InputLabel>Filter</InputLabel>
          <Select label="Filter" value={filter} onChange={(e) => setFilter(e.target.value as typeof filter)}>
            <MenuItem value="all">All</MenuItem>
            <MenuItem value="learning">Learning</MenuItem>
            <MenuItem value="learned">Learned</MenuItem>
          </Select>
        </FormControl>
      </Box>

      {loading ? (
        <Typography color="text.secondary">Loading…</Typography>
      ) : filtered.length === 0 ? (
        <Paper sx={{ p: 4, textAlign: "center" }}>
          <Typography color="text.secondary">
            {items.length === 0
              ? "No vocabulary yet. Look up a word in the reader to add it here."
              : "No words match your search."}
          </Typography>
        </Paper>
      ) : (
        <TableContainer component={Paper} variant="outlined">
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>Term</TableCell>
                <TableCell>Lookups</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Note</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {filtered.map((it) => (
                <TableRow key={it.id} hover>
                  <TableCell sx={{ fontWeight: 600 }}>{it.term}</TableCell>
                  <TableCell>{it.lookup_count}</TableCell>
                  <TableCell>
                    <Checkbox
                      checked={it.learned}
                      onChange={async () => {
                        await updateVocabulary({ ...it, learned: !it.learned });
                        load();
                      }}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>
                    {editingId === it.id ? (
                      <TextField
                        size="small"
                        fullWidth
                        value={editNote}
                        autoFocus
                        onChange={(e) => setEditNote(e.target.value)}
                        onBlur={async () => {
                          await updateVocabulary({ ...it, note: editNote.trim() ? editNote.trim() : null });
                          setEditingId(null);
                          load();
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") (e.target as HTMLInputElement).blur();
                        }}
                      />
                    ) : (
                      <Typography variant="body2" color="text.secondary">
                        {it.note ?? ""}
                      </Typography>
                    )}
                  </TableCell>
                  <TableCell>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setEditingId(it.id);
                        setEditNote(it.note ?? "");
                      }}
                    >
                      <Edit fontSize="small" />
                    </IconButton>
                    <IconButton
                      size="small"
                      title="Open source paper"
                      onClick={() => navigate(`/read/${it.paper_id_first_seen[0]}`)}
                    >
                      <MenuBook fontSize="small" />
                    </IconButton>
                    <IconButton
                      size="small"
                      color="error"
                      onClick={async () => {
                        await deleteVocabulary(it.id);
                        load();
                      }}
                    >
                      <Delete fontSize="small" />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Box>
  );
}