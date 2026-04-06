#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextAnchors {
    pub quote: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

pub fn build_text_anchors(text: &str, start_offset: u32, end_offset: u32) -> TextAnchors {
    let start = start_offset as usize;
    let end = end_offset as usize;
    let prefix_start = start.saturating_sub(16);
    let suffix_end = (end + 16).min(text.len());

    TextAnchors {
        quote: text[start..end].to_string(),
        prefix: if prefix_start == start {
            None
        } else {
            Some(text[prefix_start..start].to_string())
        },
        suffix: if end == suffix_end {
            None
        } else {
            Some(text[end..suffix_end].to_string())
        },
    }
}
