pub mod popover;

use paper_immersive_reader_core::domain::BlockId;

#[derive(Debug, Clone)]
pub struct TextSelection {
    pub block_id: BlockId,
    pub text: String,
    pub start_offset: u32,
    pub end_offset: u32,
}

#[derive(Debug, Clone)]
pub enum SelectionAction {
    Annotate,
    Translate,
    ExplainTerm,
    ExplainSentence,
}

/// Compute prefix and suffix context around a selection within the full block text.
pub fn compute_anchors(
    block_text: &str,
    selection: &TextSelection,
) -> (Option<String>, Option<String>) {
    let chars: Vec<char> = block_text.chars().collect();
    let start = selection.start_offset as usize;
    let end = selection.end_offset as usize;

    let prefix = if start > 0 {
        let prefix_start = start.saturating_sub(50);
        Some(chars[prefix_start..start].iter().collect())
    } else {
        None
    };

    let suffix = if end < chars.len() {
        let suffix_end = (end + 50).min(chars.len());
        Some(chars[end..suffix_end].iter().collect())
    } else {
        None
    };

    (prefix, suffix)
}
