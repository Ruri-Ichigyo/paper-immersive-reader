import React, { useEffect, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  FormControl,
  FormControlLabel,
  InputLabel,
  MenuItem,
  Select,
  Switch,
  TextField,
  Typography,
} from "@mui/material";
import { useApp } from "../context/AppContext";
import type { Settings } from "../types";
import { DEFAULT_SETTINGS } from "../types";

export default function SettingsPage() {
  const { settings, saveSettings, gatewayClient } = useApp();
  const [form, setForm] = useState<Settings>(settings);
  const [gatewayStatus, setGatewayStatus] = useState<boolean | null>(null);
  const [checking, setChecking] = useState(false);
  const [saved, setSaved] = useState(false);
  const [newTag, setNewTag] = useState("");

  useEffect(() => {
    setForm(settings);
  }, [settings]);

  const handleSave = async () => {
    await saveSettings(form);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const checkGateway = async () => {
    setChecking(true);
    const ok = await gatewayClient.health();
    setGatewayStatus(ok);
    setChecking(false);
  };

  return (
    <Box sx={{ maxWidth: 600 }}>
      <Typography variant="h5" sx={{ mb: 3 }}>
        Settings
      </Typography>

      <Card variant="outlined" sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="subtitle1" sx={{ fontWeight: 600,  mb: 2 }}>
            AI & Translation
          </Typography>

          <FormControl fullWidth size="small" sx={{ mb: 2 }}>
            <InputLabel>Explanation Language</InputLabel>
            <Select
              label="Explanation Language"
              value={form.explanation_language}
              onChange={(e) => setForm({ ...form, explanation_language: e.target.value })}
            >
              <MenuItem value="en">English</MenuItem>
              <MenuItem value="zh">中文</MenuItem>
              <MenuItem value="ja">日本語</MenuItem>
              <MenuItem value="ko">한국어</MenuItem>
            </Select>
          </FormControl>

          <TextField
            label="Gateway URL"
            fullWidth
            size="small"
            value={form.ai_gateway_url ?? ""}
            onChange={(e) => setForm({ ...form, ai_gateway_url: e.target.value })}
            placeholder="http://127.0.0.1:3000"
            sx={{ mb: 2 }}
          />

          <TextField
            label="Gateway Token (optional)"
            fullWidth
            size="small"
            type="password"
            value={form.ai_gateway_token_ref ?? ""}
            onChange={(e) => setForm({ ...form, ai_gateway_token_ref: e.target.value })}
            sx={{ mb: 2 }}
          />

          <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
            <Button variant="outlined" onClick={checkGateway} disabled={checking}>
              {checking ? <CircularProgress size={16} /> : "Check Gateway"}
            </Button>
            {checking && <Typography variant="body2" color="text.secondary">Checking…</Typography>}
            {!checking && gatewayStatus === true && (
              <Chip label="Gateway online" color="success" size="small" />
            )}
            {!checking && gatewayStatus === false && (
              <Chip label="Gateway offline" color="error" size="small" />
            )}
          </Box>
        </CardContent>
      </Card>

      <Card variant="outlined" sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="subtitle1" sx={{ fontWeight: 600,  mb: 2 }}>
            Custom Tags
          </Typography>
          <Box sx={{ display: "flex", gap: 1, mb: 1 }}>
            <TextField
              label="Add tag"
              size="small"
              value={newTag}
              onChange={(e) => setNewTag(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && newTag.trim()) {
                  setForm({
                    ...form,
                    custom_tags: [...Array.from(new Set([...form.custom_tags, newTag.trim()]))],
                  });
                  setNewTag("");
                }
              }}
            />
            <Button
              variant="outlined"
              onClick={() => {
                if (newTag.trim())
                  setForm({
                    ...form,
                    custom_tags: [...Array.from(new Set([...form.custom_tags, newTag.trim()]))],
                  });
                setNewTag("");
              }}
            >
              Add
            </Button>
          </Box>
          <Box sx={{ display: "flex", gap: 0.5, flexWrap: "wrap" }}>
            {form.custom_tags.map((t) => (
              <Chip
                key={t}
                label={t}
                onDelete={() =>
                  setForm({ ...form, custom_tags: form.custom_tags.filter((x) => x !== t) })
                }
              />
            ))}
          </Box>
        </CardContent>
      </Card>

      <Card variant="outlined" sx={{ mb: 3 }}>
        <CardContent>
          <FormControlLabel
            control={
              <Switch
                checked={form.sidebar_collapsed}
                onChange={(e) => setForm({ ...form, sidebar_collapsed: e.target.checked })}
              />
            }
            label="Collapse reader sidebar by default"
          />
        </CardContent>
      </Card>

      <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
        <Button variant="contained" onClick={handleSave}>
          Save Settings
        </Button>
        {saved && <Alert severity="success" sx={{ py: 0 }}>Saved</Alert>}
        <Button onClick={() => setForm(DEFAULT_SETTINGS)} color="inherit">
          Reset
        </Button>
      </Box>
    </Box>
  );
}