import React from "react";
import { Box, Button, Tooltip } from "@mui/material";
import { StickyNote2, Translate, Lightbulb, MenuBook, Close } from "@mui/icons-material";

export type SelectionActionKind = "note" | "translate" | "term" | "sentence";

interface Props {
  selection: { text: string } | null;
  onAction: (kind: SelectionActionKind) => void;
  onDismiss: () => void;
}

export default function TextSelectionPopover({ selection, onAction, onDismiss }: Props) {
  if (!selection) return null;
  return (
    <Box
      onMouseUp={(e) => e.stopPropagation()}
      onMouseDown={(e) => e.stopPropagation()}
      sx={{
        position: "fixed",
        bottom: 24,
        left: "50%",
        transform: "translateX(-50%)",
        zIndex: 1300,
        display: "flex",
        gap: 0.5,
        alignItems: "center",
        bgcolor: "background.paper",
        border: 1,
        borderColor: "primary.main",
        borderRadius: 2,
        boxShadow: 4,
        px: 1,
        py: 0.5,
      }}
    >
      <Button size="small" startIcon={<StickyNote2 />} onClick={() => onAction("note")}>
        Note
      </Button>
      <Button size="small" startIcon={<Translate />} onClick={() => onAction("translate")}>
        Translate
      </Button>
      <Button size="small" startIcon={<Lightbulb />} onClick={() => onAction("term")}>
        Term
      </Button>
      <Button size="small" startIcon={<MenuBook />} onClick={() => onAction("sentence")}>
        Sentence
      </Button>
      <Tooltip title="Dismiss">
        <Button size="small" color="inherit" onClick={onDismiss}>
          <Close fontSize="small" />
        </Button>
      </Tooltip>
    </Box>
  );
}