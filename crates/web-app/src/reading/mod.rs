pub mod navigation;
pub mod session;

pub use session::{ReadingSession, ReadingSessionError};

use paper_immersive_reader_core::domain::{BlockId, PaperId};

use crate::components::annotation_renderer::RenderedAnnotation;
use crate::components::block_navigator::BlockNavigator;
use crate::components::progress_indicator::ProgressIndicator;
use crate::components::selection_popover::SelectionPopoverView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadingView {
    pub paper_id: PaperId,
    pub current_block_id: BlockId,
    pub current_block_index: u32,
    pub block_text: String,
    pub progress_indicator: ProgressIndicator,
    pub block_navigator: BlockNavigator,
    pub selection_popover: Option<SelectionPopoverView>,
    pub annotations: Vec<RenderedAnnotation>,
}
