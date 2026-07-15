use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    num::NonZeroUsize,
};

use ctx_history_store::{BoundedBulkWriteTransaction, BULK_WRITE_BATCH_UNITS};
use serde::Serialize;

use super::*;

pub(crate) const IMPORT_TRANSACTION_BATCH_UNITS: usize = BULK_WRITE_BATCH_UNITS;

pub(super) fn import_normalized_provider_captures(
    store: &mut Store,
    normalization: ProviderNormalizationResult,
    options: NormalizedProviderImportOptions,
) -> Result<ProviderImportSummary> {
    let transaction_batch_size = provider_transaction_batch_size();
    let ProviderNormalizationResult {
        summary,
        captures,
        files_touched,
    } = normalization;
    import_provider_capture_lines_with_batch_size(
        store,
        options,
        summary,
        captures,
        files_touched,
        transaction_batch_size,
        true,
    )
}

#[cfg(test)]
pub(crate) fn import_normalized_provider_captures_in_batches(
    store: &mut Store,
    normalization: ProviderNormalizationResult,
    options: NormalizedProviderImportOptions,
    transaction_batch_size: usize,
) -> Result<ProviderImportSummary> {
    if !options.wrap_transaction {
        return Err(CaptureError::InvalidPayload(
            "batched provider import requires transaction wrapping".to_owned(),
        ));
    }
    let transaction_batch_size = NonZeroUsize::new(transaction_batch_size).ok_or_else(|| {
        CaptureError::InvalidPayload(
            "provider import batch size must be greater than zero".to_owned(),
        )
    })?;
    let ProviderNormalizationResult {
        summary,
        captures,
        files_touched,
    } = normalization;
    import_provider_capture_lines_with_batch_size(
        store,
        options,
        summary,
        captures,
        files_touched,
        Some(transaction_batch_size),
        true,
    )
}

pub(super) fn import_provider_capture_lines(
    store: &mut Store,
    options: NormalizedProviderImportOptions,
    summary: ProviderImportSummary,
    captures: Vec<(usize, ProviderCaptureEnvelope)>,
    files_touched: Vec<(usize, ProviderFileTouchedEnvelope)>,
) -> Result<ProviderImportSummary> {
    import_provider_capture_lines_with_batch_size(
        store,
        options,
        summary,
        captures,
        files_touched,
        provider_transaction_batch_size(),
        true,
    )
}

fn provider_transaction_batch_size() -> Option<NonZeroUsize> {
    NonZeroUsize::new(IMPORT_TRANSACTION_BATCH_UNITS)
}

fn import_provider_capture_lines_with_batch_size(
    store: &mut Store,
    options: NormalizedProviderImportOptions,
    mut summary: ProviderImportSummary,
    mut captures: Vec<(usize, ProviderCaptureEnvelope)>,
    mut files_touched: Vec<(usize, ProviderFileTouchedEnvelope)>,
    transaction_batch_size: Option<NonZeroUsize>,
    suppress_search_merges: bool,
) -> Result<ProviderImportSummary> {
    let caches = ProviderImportCaches::default();
    filter_provider_capture_lines_without_real_session_messages(
        &mut summary,
        &mut captures,
        &mut files_touched,
    );
    let supplied_file_touch_lines = files_touched
        .iter()
        .map(|(line_number, _)| *line_number)
        .collect::<BTreeSet<_>>();
    if summary.failed == 0 && !provider_capture_lines_have_real_message(&captures) {
        let line = captures
            .first()
            .map(|(line_number, _)| *line_number)
            .or_else(|| files_touched.first().map(|(line_number, _)| *line_number))
            .unwrap_or(0);
        summary.failed += 1;
        summary.failures.push(ProviderImportFailure {
            line,
            error: "provider source contained no real conversation message".to_owned(),
        });
        return Ok(summary);
    }
    for (line_number, capture) in &captures {
        if capture.provider == CaptureProvider::Codex
            || supplied_file_touch_lines.contains(line_number)
        {
            continue;
        }
        if let Some(event) = &capture.event {
            files_touched.extend(provider_file_touches_from_event(
                capture.provider,
                &capture.session.provider_session_id,
                &capture.source.source_format,
                capture.source.raw_source_path.as_deref(),
                capture.source.source_root.as_deref(),
                event,
                *line_number,
            ));
        }
    }
    let has_captures = !captures.is_empty() || !files_touched.is_empty();
    let bulk_search_mode = suppress_search_merges && has_captures && options.wrap_transaction;
    let bulk_search_guard = bulk_search_mode
        .then(|| store.begin_event_search_bulk_mode())
        .transpose()?;
    let import_result = persist_provider_capture_lines(
        store,
        &options,
        summary,
        captures,
        files_touched,
        has_captures,
        transaction_batch_size,
        caches,
    );
    let finish_result = match &bulk_search_guard {
        Some(guard) => store
            .defer_event_search_bulk_mode(guard)
            .map_err(CaptureError::from),
        None => Ok(()),
    };
    match (import_result, finish_result) {
        (Ok(summary), Ok(())) => Ok(summary),
        (_, Err(err)) => Err(err),
        (Err(err), Ok(())) => Err(err),
    }
}

#[allow(clippy::too_many_arguments)]
fn persist_provider_capture_lines(
    store: &mut Store,
    options: &NormalizedProviderImportOptions,
    mut summary: ProviderImportSummary,
    captures: Vec<(usize, ProviderCaptureEnvelope)>,
    files_touched: Vec<(usize, ProviderFileTouchedEnvelope)>,
    has_captures: bool,
    transaction_batch_size: Option<NonZeroUsize>,
    mut caches: ProviderImportCaches,
) -> Result<ProviderImportSummary> {
    let pending_cursors = if options.persist_cursors && summary.failed == 0 {
        captures
            .iter()
            .filter_map(|(_, capture)| provider_sync_cursor(capture))
            .map(|cursor| (cursor.id, cursor))
            .collect::<BTreeMap<_, _>>()
    } else {
        BTreeMap::new()
    };
    let mut transaction = ProviderImportTransaction::begin(
        store,
        has_captures && options.wrap_transaction,
        transaction_batch_size,
    )?;
    for (line_number, capture) in captures {
        let unit_bytes = serialized_len_or_rollback(&mut transaction, store, &capture)?;
        transaction.prepare_unit(store, unit_bytes)?;
        match import_provider_capture_line(store, &capture, options, line_number, &mut caches) {
            Ok(line_summary) => summary.merge(line_summary),
            Err(err @ CaptureError::Store(_)) => {
                transaction.rollback(store);
                return Err(err);
            }
            Err(err) => {
                summary.failed += 1;
                summary.failures.push(ProviderImportFailure {
                    line: line_number,
                    error: err.to_string(),
                });
            }
        }
        transaction.record_unit(store, unit_bytes)?;
    }
    resolve_pending_provider_edges_batched(store, &mut summary, &mut caches, &mut transaction)?;
    for (line_number, file) in files_touched {
        let unit_bytes = serialized_len_or_rollback(&mut transaction, store, &file)?;
        transaction.prepare_unit(store, unit_bytes)?;
        match import_provider_file_touched_line(store, &file, options) {
            Ok(()) => summary.accepted_content_records += 1,
            Err(err) => match err {
                err @ CaptureError::Store(_) => {
                    transaction.rollback(store);
                    return Err(err);
                }
                err => {
                    summary.failed += 1;
                    summary.failures.push(ProviderImportFailure {
                        line: line_number,
                        error: err.to_string(),
                    });
                }
            },
        }
        transaction.record_unit(store, unit_bytes)?;
    }
    if summary.failed == 0 {
        for cursor in pending_cursors.into_values() {
            let unit_bytes = serialized_len_or_rollback(&mut transaction, store, &cursor)?;
            transaction.prepare_unit(store, unit_bytes)?;
            if let Err(err) = persist_provider_sync_cursor(store, &cursor) {
                transaction.rollback(store);
                return Err(err);
            }
            transaction.record_unit(store, unit_bytes)?;
        }
    }
    transaction.commit(store)?;
    Ok(summary)
}

fn serialized_len(value: &impl Serialize) -> Result<usize> {
    let mut counter = ByteCounter::default();
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.bytes)
}

fn serialized_len_or_rollback(
    transaction: &mut ProviderImportTransaction,
    store: &Store,
    value: &impl Serialize,
) -> Result<usize> {
    match serialized_len(value) {
        Ok(bytes) => Ok(bytes),
        Err(err) => {
            transaction.rollback(store);
            Err(err)
        }
    }
}

fn pending_edge_estimated_len(edge: &PendingProviderEdge) -> usize {
    edge.provider_session_id
        .len()
        .saturating_add(
            edge.parent_provider_session_id
                .as_deref()
                .map_or(0, str::len),
        )
        .saturating_add(edge.source_format.len())
        .saturating_add(256)
}

pub(crate) fn resolve_pending_provider_edges_batched(
    store: &mut Store,
    summary: &mut ProviderImportSummary,
    caches: &mut ProviderImportCaches,
    transaction: &mut ProviderImportTransaction,
) -> Result<()> {
    let pending = std::mem::take(&mut caches.pending_edges);
    for (edge_id, edge) in pending {
        let unit_bytes = pending_edge_estimated_len(&edge);
        transaction.prepare_unit(store, unit_bytes)?;
        if let Err(err) = resolve_pending_provider_edge(store, summary, caches, edge_id, edge) {
            transaction.rollback(store);
            return Err(err);
        }
        transaction.record_unit(store, unit_bytes)?;
    }
    Ok(())
}

#[derive(Default)]
struct ByteCounter {
    bytes: usize,
}

impl Write for ByteCounter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.bytes = self.bytes.saturating_add(buffer.len());
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) type ProviderImportTransaction = BoundedBulkWriteTransaction;
