use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use ctx_history_store::Store;

use crate::provider::adapter::{
    AuggieSessionJsonAdapter, ClaudeProjectsJsonlAdapter, ClineTaskJsonAdapter,
    CodeBuddyHistoryJsonAdapter, CrushSqliteAdapter, GooseSessionsSqliteAdapter,
    HermesSqliteAdapter, JunieSessionEventsAdapter, OpenClawJsonlAdapter, PiSessionJsonlAdapter,
    ProviderCaptureAdapter, RooTaskJsonAdapter,
};
use crate::provider::importer::{
    import_native_jsonl_tree, import_normalized_provider_captures,
    import_normalized_provider_captures_stream_batch,
    import_normalized_provider_captures_with_progress, NativeJsonlTreeImport,
    ProviderImportStreamState,
};
use crate::provider::providers::claude::{
    import_large_claude_projects_jsonl_file_streaming, normalize_claude_projects_jsonl_file,
    normalize_claude_projects_jsonl_paths_parallel,
};
use crate::provider::providers::trae::normalize_trae_history;
use crate::{
    AuggieImportOptions, ClaudeProjectsImportOptions, ClineTaskJsonImportOptions,
    CodeBuddyImportOptions, CrushSqliteImportOptions, GooseSessionsSqliteImportOptions,
    HermesSqliteImportOptions, JunieImportOptions, NormalizedProviderImportOptions,
    OpenClawImportOptions, PiSessionImportOptions, ProviderAdapterContext, ProviderImportSummary,
    Result, RooTaskJsonImportOptions, TraeImportOptions,
};

pub fn import_pi_session_jsonl(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: PiSessionImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = PiSessionJsonlAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;

    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}

pub fn import_claude_projects_jsonl_tree(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: ClaudeProjectsImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let context = ProviderAdapterContext {
        machine_id: options.machine_id,
        source_path: Some(source_path),
        source_root: None,
        imported_at: options.imported_at,
    };
    let normalization = if path.is_file() {
        normalize_claude_projects_jsonl_file(path, &context, options.progress.as_ref())?
    } else {
        ClaudeProjectsJsonlAdapter.normalize_path(path, &context)?
    };

    let summary = import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )?;
    if let Some(callback) = options.progress {
        let total_bytes = std::fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        callback(crate::ProviderImportProgress {
            stage: crate::ProviderImportStage::Writing,
            source_path: Some(path.to_path_buf()),
            total_files: 1,
            total_bytes,
            completed_files: 1,
            completed_bytes: total_bytes,
            completed_units: summary.imported_events,
            total_units: summary.imported_events,
            imported_sessions: summary.imported_sessions,
            imported_events: summary.imported_events,
            imported_edges: summary.imported_edges,
            skipped: summary.skipped,
            failed: summary.failed,
            done: true,
        });
    }
    Ok(summary)
}

/// Uses parallel workers only for CPU-bound transcript normalization; all
/// SQLite writes remain on the caller thread in deterministic path order.
pub fn import_claude_projects_jsonl_files_bounded_parallel(
    store: &mut Store,
    paths: &[PathBuf],
    options: &ClaudeProjectsImportOptions,
) -> Result<Vec<(PathBuf, ProviderImportSummary)>> {
    const NORMALIZATION_BATCH_BYTES: u64 = 64 * 1024 * 1024;
    let context = ProviderAdapterContext {
        machine_id: options.machine_id.clone(),
        source_path: options.source_path.clone(),
        source_root: None,
        imported_at: options.imported_at,
    };
    let parallelism = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(8);
    let mut imported = Vec::with_capacity(paths.len());
    let mut batch = Vec::new();
    let mut batch_bytes = 0u64;
    for path in paths {
        let bytes = std::fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        if bytes >= NORMALIZATION_BATCH_BYTES {
            if !batch.is_empty() {
                import_claude_normalization_batch(
                    store,
                    &batch,
                    &context,
                    parallelism,
                    options,
                    &mut imported,
                )?;
                batch.clear();
                batch_bytes = 0;
            }
            let summary = import_large_claude_file_streaming(store, path, &context, options)?;
            imported.push((path.clone(), summary));
            continue;
        }
        if !batch.is_empty() && batch_bytes.saturating_add(bytes) > NORMALIZATION_BATCH_BYTES {
            import_claude_normalization_batch(
                store,
                &batch,
                &context,
                parallelism,
                options,
                &mut imported,
            )?;
            batch.clear();
            batch_bytes = 0;
        }
        batch.push(path.clone());
        batch_bytes = batch_bytes.saturating_add(bytes);
    }
    if !batch.is_empty() {
        import_claude_normalization_batch(
            store,
            &batch,
            &context,
            parallelism,
            options,
            &mut imported,
        )?;
    }
    Ok(imported)
}

fn import_large_claude_file_streaming(
    store: &mut Store,
    path: &Path,
    context: &ProviderAdapterContext,
    options: &ClaudeProjectsImportOptions,
) -> Result<ProviderImportSummary> {
    let file_bytes = std::fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let progress_callback = options.progress.as_ref().map(Arc::clone);
    let progress_path = path.to_path_buf();
    let mut persisted_units = 0usize;
    let mut stream_state = ProviderImportStreamState::default();
    import_large_claude_projects_jsonl_file_streaming(
        path,
        context,
        options.progress.as_ref(),
        // Amortize SQLite transaction/checkpoint overhead while keeping the
        // normalized working set bounded for multi-gigabyte transcripts.
        16 * 1024 * 1024,
        |normalization| {
            let batch_units = normalization
                .captures
                .len()
                .saturating_add(normalization.files_touched.len());
            let persisted_before_batch = persisted_units;
            let mut persist_progress = |completed_units: usize, _total_units: usize| {
                if let Some(callback) = progress_callback.as_ref() {
                    callback(crate::ProviderImportProgress {
                        stage: crate::ProviderImportStage::Writing,
                        source_path: Some(progress_path.clone()),
                        total_files: 1,
                        total_bytes: file_bytes,
                        completed_files: 0,
                        completed_bytes: 0,
                        completed_units: persisted_before_batch.saturating_add(completed_units),
                        // The total is unknown until the complete JSONL file
                        // has been normalized. Zero explicitly means an
                        // indeterminate writer total, not zero work.
                        total_units: 0,
                        imported_sessions: 0,
                        imported_events: 0,
                        imported_edges: 0,
                        skipped: 0,
                        failed: 0,
                        done: false,
                    });
                }
            };
            let result = import_normalized_provider_captures_stream_batch(
                store,
                normalization,
                NormalizedProviderImportOptions {
                    history_record_id: options.history_record_id,
                    persist_cursors: true,
                    wrap_transaction: true,
                    fast_event_inserts: true,
                },
                &mut stream_state,
                &mut persist_progress,
            );
            if result.is_ok() {
                persisted_units = persisted_units.saturating_add(batch_units);
            }
            result
        },
    )
}

fn import_claude_normalization_batch(
    store: &mut Store,
    paths: &[PathBuf],
    context: &ProviderAdapterContext,
    parallelism: usize,
    options: &ClaudeProjectsImportOptions,
    imported: &mut Vec<(PathBuf, ProviderImportSummary)>,
) -> Result<()> {
    for (_, path, normalization) in normalize_claude_projects_jsonl_paths_parallel(
        paths,
        context,
        parallelism,
        options.progress.as_ref(),
    )? {
        let file_bytes = std::fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let progress_callback = options.progress.as_ref().map(Arc::clone);
        let progress_path = path.clone();
        let mut persist_progress = move |completed_units: usize, total_units: usize| {
            if let Some(callback) = progress_callback.as_ref() {
                callback(crate::ProviderImportProgress {
                    stage: crate::ProviderImportStage::Writing,
                    source_path: Some(progress_path.clone()),
                    total_files: 1,
                    total_bytes: file_bytes,
                    completed_files: usize::from(completed_units >= total_units),
                    completed_bytes: if completed_units >= total_units {
                        file_bytes
                    } else {
                        0
                    },
                    completed_units,
                    total_units,
                    imported_sessions: 0,
                    imported_events: 0,
                    imported_edges: 0,
                    skipped: 0,
                    failed: 0,
                    done: completed_units >= total_units,
                });
            }
        };
        let summary = import_normalized_provider_captures_with_progress(
            store,
            normalization,
            NormalizedProviderImportOptions {
                history_record_id: options.history_record_id,
                persist_cursors: true,
                wrap_transaction: true,
                fast_event_inserts: true,
            },
            &mut persist_progress,
        )?;
        imported.push((path, summary));
    }
    Ok(())
}

pub fn import_cline_task_json_history(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: ClineTaskJsonImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = ClineTaskJsonAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;

    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}

pub fn import_roo_task_json_history(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: RooTaskJsonImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = RooTaskJsonAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;

    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}

pub fn import_codebuddy_history(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: CodeBuddyImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = CodeBuddyHistoryJsonAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;

    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}

pub fn import_trae_history(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: TraeImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = normalize_trae_history(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;

    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}

pub fn import_crush_sqlite(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: CrushSqliteImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = CrushSqliteAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;

    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}

pub fn import_goose_sessions_sqlite(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: GooseSessionsSqliteImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = GooseSessionsSqliteAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;

    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}

pub fn import_openclaw_history(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: OpenClawImportOptions,
) -> Result<ProviderImportSummary> {
    import_native_jsonl_tree(
        store,
        NativeJsonlTreeImport {
            path: path.as_ref(),
            machine_id: options.machine_id,
            source_path: options.source_path,
            source_root: None,
            imported_at: options.imported_at,
            history_record_id: options.history_record_id,
        },
        OpenClawJsonlAdapter,
    )
}

pub fn import_hermes_sqlite(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: HermesSqliteImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = HermesSqliteAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;
    let import_options = NormalizedProviderImportOptions {
        history_record_id: options.history_record_id,
        persist_cursors: true,
        wrap_transaction: true,
        fast_event_inserts: true,
    };
    import_normalized_provider_captures(store, normalization, import_options)
}

pub fn import_auggie_history(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: AuggieImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = AuggieSessionJsonAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;

    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}

pub fn import_junie_history(
    path: impl AsRef<Path>,
    store: &mut Store,
    options: JunieImportOptions,
) -> Result<ProviderImportSummary> {
    let path = path.as_ref();
    let source_path = options
        .source_path
        .clone()
        .unwrap_or_else(|| path.to_path_buf());
    let normalization = JunieSessionEventsAdapter.normalize_path(
        path,
        &ProviderAdapterContext {
            machine_id: options.machine_id,
            source_path: Some(source_path),
            source_root: None,
            imported_at: options.imported_at,
        },
    )?;
    import_normalized_provider_captures(
        store,
        normalization,
        NormalizedProviderImportOptions {
            history_record_id: options.history_record_id,
            persist_cursors: true,
            wrap_transaction: true,
            fast_event_inserts: true,
        },
    )
}
