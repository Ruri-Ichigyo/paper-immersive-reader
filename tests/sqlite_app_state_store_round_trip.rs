use paper_immersive_reader_application::persistence::SqliteAppStateStore;
use paper_immersive_reader_core::domain::{
    Block, BlockId, OcrEngine, OcrJob, OcrJobId, OcrJobResult, OcrJobStatus, Paper, PaperId,
    PaperSourceType, PaperStatus, ReadingProgress,
};
use paper_immersive_reader_importing::{ImportResult, TextImportArtifacts};

#[test]
fn sqlite_store_round_trips_current_persisted_entities() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-sqlite-round-trip");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).expect("temp root should exist");
    let db_path = temp_root.join("app-state.db");

    let imported_paper = Paper {
        id: PaperId("paper-imported".to_string()),
        title: Some("Imported".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("imported.pdf".to_string()),
        status: PaperStatus::ReadyText,
        parse_version: 1,
    };
    let imported_blocks = vec![
        Block::try_new(
            BlockId("block-1".to_string()),
            imported_paper.id.clone(),
            0,
            "first block".to_string(),
            None,
        )
        .expect("block should be valid"),
        Block::try_new(
            BlockId("block-2".to_string()),
            imported_paper.id.clone(),
            1,
            "second block".to_string(),
            Some("hash-2".to_string()),
        )
        .expect("block should be valid"),
    ];
    let imported_progress = ReadingProgress {
        paper_id: imported_paper.id.clone(),
        current_block_index: 1,
        current_block_id: Some(imported_blocks[1].id.clone()),
        updated_at_ms: 10,
    };

    let queued_paper = Paper {
        id: PaperId("paper-queued".to_string()),
        title: Some("Queued".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("queued.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    };
    let queued_job = OcrJob {
        id: OcrJobId("job-queued".to_string()),
        paper_id: queued_paper.id.clone(),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 20,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    };

    let completed_paper = Paper {
        id: PaperId("paper-completed".to_string()),
        title: Some("Completed".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("completed.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    };
    let completed_job = OcrJob {
        id: OcrJobId("job-completed".to_string()),
        paper_id: completed_paper.id.clone(),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 30,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    }
    .mark_processing(40)
    .expect("job should start")
    .mark_completed(50)
    .expect("job should complete");
    let completed_result = OcrJobResult {
        paper_id: completed_paper.id.clone(),
        text: "ocr output".to_string(),
    };
    let completed_blocks = vec![Block::try_new(
        BlockId("block-completed-1".to_string()),
        completed_paper.id.clone(),
        0,
        "ocr block".to_string(),
        None,
    )
    .expect("block should be valid")];
    let completed_progress = ReadingProgress {
        paper_id: completed_paper.id.clone(),
        current_block_index: 0,
        current_block_id: Some(completed_blocks[0].id.clone()),
        updated_at_ms: 50,
    };
    let updated_completed_paper = Paper {
        status: PaperStatus::ReadyText,
        ..completed_paper.clone()
    };

    let mut store = SqliteAppStateStore::open(&db_path).expect("sqlite store should open");
    store
        .save_import_result(ImportResult {
            paper: imported_paper.clone(),
            blocks: imported_blocks.clone(),
            reading_progress: imported_progress.clone(),
        })
        .expect("import result should save");
    store
        .save_paper_and_job(queued_paper.clone(), queued_job.clone())
        .expect("queued paper and job should save");
    store
        .save_paper_and_job(completed_paper.clone(), completed_job.clone())
        .expect("completed paper and job should seed");
    store
        .save_completed_ocr(
            completed_job.clone(),
            completed_result.clone(),
            TextImportArtifacts {
                blocks: completed_blocks.clone(),
                reading_progress: completed_progress.clone(),
            },
            updated_completed_paper.clone(),
        )
        .expect("completed OCR should save");

    let reloaded = SqliteAppStateStore::open(&db_path).expect("sqlite store should reload");

    assert_eq!(
        reloaded.get_paper(&imported_paper.id),
        Some(imported_paper.clone())
    );
    assert_eq!(
        reloaded.get_paper(&queued_paper.id),
        Some(queued_paper.clone())
    );
    assert_eq!(
        reloaded.get_paper(&updated_completed_paper.id),
        Some(updated_completed_paper.clone())
    );
    assert_eq!(reloaded.get_job(&queued_job.id), Some(queued_job.clone()));
    assert_eq!(
        reloaded.get_job(&completed_job.id),
        Some(completed_job.clone())
    );
    assert_eq!(reloaded.first_queued_job(), Some(queued_job.clone()));

    let snapshot = reloaded.snapshot().expect("snapshot should load");
    assert!(snapshot.papers.contains(&imported_paper));
    assert!(snapshot.papers.contains(&queued_paper));
    assert!(snapshot.papers.contains(&updated_completed_paper));
    assert!(snapshot.blocks.contains(&imported_blocks[0]));
    assert!(snapshot.blocks.contains(&imported_blocks[1]));
    assert!(snapshot.blocks.contains(&completed_blocks[0]));
    assert!(snapshot.reading_progress.contains(&imported_progress));
    assert!(snapshot.reading_progress.contains(&completed_progress));
    assert!(snapshot.ocr_jobs.contains(&queued_job));
    assert!(snapshot.ocr_jobs.contains(&completed_job));
    assert!(snapshot.ocr_job_results.contains(&completed_result));
}
