use paper_immersive_reader_core::domain::{
    AnchoringStatus, Annotation, AnnotationId, AnnotationType, PaperId, VocabularyId,
    VocabularyItem,
};

use crate::annotations::anchors::build_text_anchors;
use crate::components::annotation_renderer::RenderedAnnotation;
use crate::components::block_navigator::BlockNavigator;
use crate::components::progress_indicator::ProgressIndicator;
use crate::reading::navigation::NavigationDirection;
use crate::reading::{ReadingSession, ReadingSessionError, ReadingView};
use crate::selection::popover::build_selection_popover;
use crate::selection::{SelectionError, SelectionState};
use crate::storage::{StorageError, WebStorage};

use super::RouteDescriptor;

pub fn descriptor() -> RouteDescriptor {
    RouteDescriptor {
        path: "/read",
        label: "Read",
        description: "Read a paper block by block in the immersive reader shell.",
    }
}

#[derive(Debug)]
pub enum ReadRouteError {
    ReadingSession(ReadingSessionError),
    Selection(SelectionError),
    Storage(StorageError),
}

impl From<ReadingSessionError> for ReadRouteError {
    fn from(value: ReadingSessionError) -> Self {
        Self::ReadingSession(value)
    }
}

impl From<SelectionError> for ReadRouteError {
    fn from(value: SelectionError) -> Self {
        Self::Selection(value)
    }
}

impl From<StorageError> for ReadRouteError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

pub struct ReadRouteModel {
    storage: WebStorage,
    session: ReadingSession,
    selection: Option<SelectionState>,
}

impl ReadRouteModel {
    pub fn load(storage: WebStorage, paper_id: PaperId) -> Result<Self, ReadingSessionError> {
        Ok(Self {
            session: ReadingSession::load(storage.clone(), paper_id)?,
            storage,
            selection: None,
        })
    }

    pub fn go_next(&mut self, updated_at_ms: u64) -> Result<(), ReadingSessionError> {
        self.session
            .navigate(NavigationDirection::Next, updated_at_ms)
    }

    pub fn go_previous(&mut self, updated_at_ms: u64) -> Result<(), ReadingSessionError> {
        self.session
            .navigate(NavigationDirection::Previous, updated_at_ms)
    }

    pub fn select_current_block_range(
        &mut self,
        start_offset: u32,
        end_offset: u32,
    ) -> Result<(), ReadRouteError> {
        let current_block = self.session.current_block();
        let start = start_offset as usize;
        let end = end_offset as usize;

        let quote = current_block
            .text
            .get(start..end)
            .ok_or(SelectionError::InvalidOffsets)?
            .to_string();

        self.selection = Some(SelectionState {
            paper_id: self.session.paper().id.clone(),
            block_id: current_block.id.clone(),
            quote,
            start_offset,
            end_offset,
        });

        Ok(())
    }

    pub fn save_highlight(
        &mut self,
        note: Option<String>,
        tags: Vec<String>,
    ) -> Result<Annotation, ReadRouteError> {
        let selection = self
            .selection
            .clone()
            .ok_or(SelectionError::MissingSelection)?;
        let current_block = self.session.current_block();
        let anchors = build_text_anchors(
            &current_block.text,
            selection.start_offset,
            selection.end_offset,
        );
        let annotation_count = self
            .storage
            .list_annotations_for_paper(&selection.paper_id)?
            .len();
        let annotation = Annotation {
            id: AnnotationId(format!("annotation-{}", annotation_count + 1)),
            paper_id: selection.paper_id,
            block_id: selection.block_id,
            annotation_type: AnnotationType::Highlight,
            quote: anchors.quote,
            prefix: anchors.prefix,
            suffix: anchors.suffix,
            start_offset: Some(selection.start_offset),
            end_offset: Some(selection.end_offset),
            note,
            tags,
            anchoring_status: AnchoringStatus::Anchored,
        };

        self.storage.save_annotation(annotation.clone())?;
        Ok(annotation)
    }

    pub fn record_lookup_event(
        &mut self,
        lookup_time_ms: u64,
    ) -> Result<VocabularyItem, ReadRouteError> {
        let selection = self
            .selection
            .clone()
            .ok_or(SelectionError::MissingSelection)?;
        let existing_count = self
            .storage
            .list_vocabulary_for_paper(&selection.paper_id)?
            .len();
        let item = VocabularyItem {
            id: VocabularyId(format!("vocab-{}", existing_count + 1)),
            term: selection.quote,
            paper_id_first_seen: selection.paper_id,
            block_id_first_seen: Some(selection.block_id),
            lookup_count: 1,
            first_seen_at_ms: lookup_time_ms,
            last_seen_at_ms: lookup_time_ms,
            note: None,
            learned: false,
        };

        Ok(self.storage.upsert_vocabulary_lookup(item)?)
    }

    pub fn view(&self) -> ReadingView {
        let current_block = self.session.current_block();
        let current_block_index = self.session.reading_progress().current_block_index;
        let total_blocks = self.session.blocks().len();
        let annotations = self
            .storage
            .list_annotations_for_paper(&self.session.paper().id)
            .unwrap_or_default()
            .into_iter()
            .filter(|annotation| annotation.block_id == current_block.id)
            .map(RenderedAnnotation::from)
            .collect();

        ReadingView {
            paper_id: self.session.paper().id.clone(),
            current_block_id: current_block.id.clone(),
            current_block_index,
            block_text: current_block.text.clone(),
            progress_indicator: ProgressIndicator::from_position(current_block_index, total_blocks),
            block_navigator: BlockNavigator::from_position(current_block_index, total_blocks),
            selection_popover: self.selection.as_ref().map(build_selection_popover),
            annotations,
        }
    }
}
