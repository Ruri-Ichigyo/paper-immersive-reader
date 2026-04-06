use crate::components::selection_popover::SelectionPopoverView;

use super::SelectionState;

pub fn build_selection_popover(selection: &SelectionState) -> SelectionPopoverView {
    SelectionPopoverView::from(selection)
}
