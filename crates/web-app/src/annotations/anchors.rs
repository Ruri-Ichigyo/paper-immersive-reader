use crate::selection::TextSelection;

/// Compute prefix (up to 50 chars before) and suffix (up to 50 chars after)
/// for resilient re-anchoring of annotations.
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
