use crate::selection::SelectionState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionPopoverView {
    pub selected_text: String,
    pub actions: Vec<&'static str>,
}

impl From<&SelectionState> for SelectionPopoverView {
    fn from(value: &SelectionState) -> Self {
        Self {
            selected_text: value.quote.clone(),
            actions: vec!["translate", "explain", "annotate", "save-word"],
        }
    }
}
