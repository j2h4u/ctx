use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use ctx_history_core::{
    AgentType, CaptureProvider, EventType, Fidelity, ProviderCaptureEnvelope,
    ProviderCursorCheckpoint, ProviderCursorRange, ProviderEventEnvelope, ProviderSessionEnvelope,
    ProviderSourceEnvelope, ProviderSourceTrust, SessionStatus,
    PROVIDER_CAPTURE_ENVELOPE_SCHEMA_VERSION,
};
use serde_json::{json, Value};

use crate::common::io::{
    ensure_regular_provider_transcript_file, read_provider_jsonl_record_or_skip_oversized,
};
use crate::common::time::parse_rfc3339_utc;
use crate::provider::file_touches::provider_file_touches_from_raw_value;
use crate::provider::importer::{
    provider_cursor_stream, provider_event_is_real_conversation_message,
};
use crate::provider::native::{
    provider_capped_json, provider_policy_body, provider_policy_event_text, provider_role,
    provider_value_text,
};
use crate::{
    ProviderAdapterContext, ProviderImportFailure, ProviderImportProgress,
    ProviderImportProgressCallback, ProviderImportStage, ProviderImportSummary,
    ProviderNormalizationResult, Result, CLAUDE_PROJECTS_SOURCE_FORMAT, PROVIDER_MAX_PREVIEW_CHARS,
};

pub(crate) fn normalize_claude_projects_jsonl_file(
    path: &Path,
    context: &ProviderAdapterContext,
    progress: Option<&ProviderImportProgressCallback>,
) -> Result<ProviderNormalizationResult> {
    ensure_regular_provider_transcript_file(path)?;
    let total_bytes = std::fs::metadata(path)?.len();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut result = ProviderNormalizationResult::default();
    let mut rows = Vec::new();
    let mut line = Vec::new();
    let mut line_number = 0usize;
    let mut completed_bytes = 0u64;
    let mut last_progress = Instant::now() - Duration::from_secs(1);

    while read_provider_jsonl_record_or_skip_oversized(
        &mut reader,
        &mut line,
        &mut line_number,
        &mut result.summary,
    )? {
        completed_bytes = completed_bytes
            .saturating_add(line.len() as u64)
            .min(total_bytes);
        if last_progress.elapsed() >= Duration::from_millis(250) {
            report_claude_progress(progress, path, total_bytes, completed_bytes, false);
            last_progress = Instant::now();
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let value: Value = match serde_json::from_slice(&line) {
            Ok(value) => value,
            Err(err) => {
                result.summary.failed += 1;
                result.summary.failures.push(ProviderImportFailure {
                    line: line_number,
                    error: format!("malformed JSONL in {}: {err}", path.display()),
                });
                continue;
            }
        };
        let timestamp = value
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(parse_rfc3339_utc)
            .unwrap_or(context.imported_at);
        rows.push((line_number, value, timestamp));
    }
    report_claude_progress(progress, path, total_bytes, completed_bytes, false);
    if rows.is_empty() {
        if result.summary.failed == 0 {
            result.summary.skipped += 1;
            result.summary.skipped_sessions += 1;
        }
        return Ok(result);
    }

    let metadata = ClaudeSessionMetadata::from_first_row(path, &rows[0].1);
    let started_at = rows
        .iter()
        .map(|(_, _, timestamp)| *timestamp)
        .min()
        .unwrap_or(context.imported_at);

    for (line_number, value, occurred_at) in rows {
        let (capture, files_touched) = claude_normalized_capture(
            path,
            context,
            &metadata,
            started_at,
            line_number,
            &value,
            occurred_at,
        );
        result.files_touched.extend(files_touched);
        result.captures.push((line_number, capture));
    }

    Ok(result)
}

#[derive(Clone)]
struct ClaudeSessionMetadata {
    native_session_id: String,
    provider_session_id: String,
    parent_provider_session_id: Option<String>,
    external_agent_id: Option<String>,
    is_subagent: bool,
    cwd: Option<String>,
    version: Option<String>,
    git_branch: Option<String>,
    raw_source_path: String,
}

impl ClaudeSessionMetadata {
    fn from_first_row(path: &Path, first: &Value) -> Self {
        let file_stem = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown-session");
        let native_session_id = first
            .get("sessionId")
            .and_then(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .unwrap_or(file_stem)
            .to_owned();
        let (provider_session_id, parent_provider_session_id, external_agent_id, is_subagent) =
            claude_path_session_ids(path, &native_session_id);
        Self {
            native_session_id,
            provider_session_id,
            parent_provider_session_id,
            external_agent_id,
            is_subagent,
            cwd: first
                .get("cwd")
                .and_then(Value::as_str)
                .filter(|cwd| !cwd.trim().is_empty())
                .map(str::to_owned),
            version: first
                .get("version")
                .and_then(Value::as_str)
                .map(str::to_owned),
            git_branch: first
                .get("gitBranch")
                .and_then(Value::as_str)
                .map(str::to_owned),
            raw_source_path: path.display().to_string(),
        }
    }
}

fn claude_normalized_capture(
    path: &Path,
    context: &ProviderAdapterContext,
    metadata: &ClaudeSessionMetadata,
    started_at: DateTime<Utc>,
    line_number: usize,
    value: &Value,
    occurred_at: DateTime<Utc>,
) -> (
    ProviderCaptureEnvelope,
    Vec<(usize, crate::ProviderFileTouchedEnvelope)>,
) {
    let event = claude_event(value, line_number, occurred_at);
    let files_touched = event.as_ref().map_or_else(Vec::new, |event| {
        provider_file_touches_from_raw_value(
            CaptureProvider::Claude,
            &metadata.provider_session_id,
            CLAUDE_PROJECTS_SOURCE_FORMAT,
            Some(metadata.raw_source_path.as_str()),
            value,
            event,
            line_number,
        )
    });
    let capture = ProviderCaptureEnvelope {
        schema_version: PROVIDER_CAPTURE_ENVELOPE_SCHEMA_VERSION,
        provider: CaptureProvider::Claude,
        source: ProviderSourceEnvelope {
            source_format: CLAUDE_PROJECTS_SOURCE_FORMAT.to_owned(),
            machine_id: context.machine_id.clone(),
            observed_at: context.imported_at,
            raw_source_path: Some(metadata.raw_source_path.clone()),
            source_root: context
                .source_root_display()
                .or_else(|| Some(metadata.raw_source_path.clone())),
            trust: ProviderSourceTrust::ProviderNative,
            fidelity: Fidelity::Imported,
            cursor: Some(ProviderCursorRange {
                before: None,
                after: Some(ProviderCursorCheckpoint {
                    stream: provider_cursor_stream(
                        CaptureProvider::Claude,
                        CLAUDE_PROJECTS_SOURCE_FORMAT,
                    ),
                    cursor: format!("{}:line:{line_number}", path.display()),
                    observed_at: occurred_at,
                }),
            }),
            idempotency_key: Some(format!(
                "provider-source:claude:{CLAUDE_PROJECTS_SOURCE_FORMAT}:{}",
                metadata.provider_session_id
            )),
            metadata: json!({
                "adapter": CLAUDE_PROJECTS_SOURCE_FORMAT,
                "native_session_id": metadata.native_session_id,
                "source_path": metadata.raw_source_path,
            }),
        },
        session: ProviderSessionEnvelope {
            provider_session_id: metadata.provider_session_id.clone(),
            parent_provider_session_id: metadata.parent_provider_session_id.clone(),
            root_provider_session_id: metadata.parent_provider_session_id.clone(),
            external_agent_id: metadata.external_agent_id.clone(),
            agent_type: if metadata.is_subagent {
                AgentType::Subagent
            } else {
                AgentType::Primary
            },
            role_hint: Some(
                if metadata.is_subagent {
                    "subagent"
                } else {
                    "primary"
                }
                .to_owned(),
            ),
            is_primary: !metadata.is_subagent,
            status: SessionStatus::Imported,
            started_at,
            ended_at: None,
            cwd: metadata.cwd.clone(),
            fidelity: Fidelity::Imported,
            idempotency_key: Some(format!(
                "provider-session:claude:{}",
                metadata.provider_session_id
            )),
            artifacts: Vec::new(),
            metadata: json!({
                "source_format": CLAUDE_PROJECTS_SOURCE_FORMAT,
                "native_session_id": metadata.native_session_id,
                "version": metadata.version,
                "git_branch": metadata.git_branch,
                "source_path": metadata.raw_source_path,
                "limitations": [
                    "binary attachments are referenced by native payload metadata but not expanded",
                    "previews are capped before local indexing/export"
                ],
            }),
        },
        event,
    };
    (capture, files_touched)
}

/// Parses and persists one large Claude transcript in bounded batches. Unlike
/// the multi-file parallel path, this never retains the complete JSON `Value`
/// tree and the complete normalized capture tree at the same time.
pub(crate) fn import_large_claude_projects_jsonl_file_streaming(
    path: &Path,
    context: &ProviderAdapterContext,
    progress: Option<&ProviderImportProgressCallback>,
    stream_batch_input_bytes: u64,
    mut import_batch: impl FnMut(ProviderNormalizationResult) -> Result<ProviderImportSummary>,
) -> Result<ProviderImportSummary> {
    ensure_regular_provider_transcript_file(path)?;
    let total_bytes = std::fs::metadata(path)?.len();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    let mut line_number = 0usize;
    let mut completed_bytes = 0u64;
    let mut batch_bytes = 0u64;
    let mut last_progress = Instant::now() - Duration::from_secs(1);
    let mut metadata = None;
    let mut started_at = context.imported_at;
    let mut saw_timestamp = false;
    let mut saw_real_message = false;
    let mut batch = ProviderNormalizationResult::default();
    let mut parse_summary = ProviderImportSummary::default();
    let mut imported_summary = ProviderImportSummary::default();

    while read_provider_jsonl_record_or_skip_oversized(
        &mut reader,
        &mut line,
        &mut line_number,
        &mut parse_summary,
    )? {
        completed_bytes = completed_bytes
            .saturating_add(line.len() as u64)
            .min(total_bytes);
        batch_bytes = batch_bytes.saturating_add(line.len() as u64);
        if last_progress.elapsed() >= Duration::from_millis(250) {
            report_claude_progress(progress, path, total_bytes, completed_bytes, false);
            last_progress = Instant::now();
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let value: Value = match serde_json::from_slice(&line) {
            Ok(value) => value,
            Err(err) => {
                parse_summary.failed += 1;
                parse_summary.failures.push(ProviderImportFailure {
                    line: line_number,
                    error: format!("malformed JSONL in {}: {err}", path.display()),
                });
                continue;
            }
        };
        let occurred_at = value
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(parse_rfc3339_utc)
            .unwrap_or(context.imported_at);
        if !saw_timestamp || occurred_at < started_at {
            started_at = occurred_at;
            saw_timestamp = true;
        }
        let metadata =
            metadata.get_or_insert_with(|| ClaudeSessionMetadata::from_first_row(path, &value));
        let (capture, files_touched) = claude_normalized_capture(
            path,
            context,
            metadata,
            started_at,
            line_number,
            &value,
            occurred_at,
        );
        saw_real_message |= capture
            .event
            .as_ref()
            .is_some_and(provider_event_is_real_conversation_message);
        batch.files_touched.extend(files_touched);
        batch.captures.push((line_number, capture));

        if saw_real_message && batch_bytes >= stream_batch_input_bytes.max(1) {
            let imported = import_batch(std::mem::take(&mut batch))?;
            imported_summary.merge_from(imported);
            batch_bytes = 0;
        }
    }
    report_claude_progress(progress, path, total_bytes, completed_bytes, true);

    if metadata.is_none() {
        if parse_summary.failed == 0 {
            parse_summary.skipped += 1;
            parse_summary.skipped_sessions += 1;
        }
        return Ok(parse_summary);
    }
    if !saw_real_message {
        parse_summary.failed += 1;
        parse_summary.failures.push(ProviderImportFailure {
            line: 0,
            error: "provider source contained no real conversation message".to_owned(),
        });
        return Ok(parse_summary);
    }
    if !batch.captures.is_empty() || !batch.files_touched.is_empty() {
        let imported = import_batch(batch)?;
        imported_summary.merge_from(imported);
    }
    imported_summary.merge_from(parse_summary);
    Ok(imported_summary)
}

/// Normalizes independent Claude transcripts concurrently while preserving a
/// deterministic order for the single SQLite writer that consumes the result.
/// Callers must keep the input chunk bounded: normalization retains captures
/// until that chunk is written.
pub(crate) fn normalize_claude_projects_jsonl_paths_parallel(
    paths: &[PathBuf],
    context: &ProviderAdapterContext,
    parallelism: usize,
    progress: Option<&ProviderImportProgressCallback>,
) -> Result<Vec<(usize, PathBuf, ProviderNormalizationResult)>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    if parallelism <= 1 || paths.len() == 1 {
        return paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                Ok((
                    index,
                    path.clone(),
                    normalize_claude_projects_jsonl_file(path, context, progress)?,
                ))
            })
            .collect();
    }

    let worker_count = parallelism.min(paths.len());
    let paths_per_worker = paths.len().div_ceil(worker_count);
    let mut batches = thread::scope(|scope| {
        let mut handles = Vec::new();
        for (worker_index, chunk) in paths.chunks(paths_per_worker).enumerate() {
            let chunk = chunk.to_vec();
            handles.push(scope.spawn(move || {
                let base_index = worker_index * paths_per_worker;
                chunk
                    .iter()
                    .enumerate()
                    .map(|(offset, path)| {
                        Ok((
                            base_index + offset,
                            path.clone(),
                            normalize_claude_projects_jsonl_file(path, context, progress)?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()
            }));
        }
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| crate::CaptureError::WorkerPanicked("Claude import"))?
            })
            .collect::<Result<Vec<_>>>()
    })?;
    let total = batches.iter().map(Vec::len).sum();
    let mut normalized = Vec::with_capacity(total);
    for batch in batches.drain(..) {
        normalized.extend(batch);
    }
    normalized.sort_by_key(|(index, _, _)| *index);
    Ok(normalized)
}

fn report_claude_progress(
    progress: Option<&ProviderImportProgressCallback>,
    path: &Path,
    total_bytes: u64,
    completed_bytes: u64,
    done: bool,
) {
    let Some(callback) = progress else {
        return;
    };
    callback(ProviderImportProgress {
        stage: ProviderImportStage::Reading,
        source_path: Some(path.to_path_buf()),
        total_files: 1,
        total_bytes,
        completed_files: usize::from(done),
        completed_bytes: if done {
            total_bytes
        } else {
            completed_bytes.min(total_bytes)
        },
        completed_units: 0,
        total_units: 0,
        imported_sessions: 0,
        imported_events: 0,
        imported_edges: 0,
        skipped: 0,
        failed: 0,
        done,
    });
}

#[cfg(test)]
mod streaming_tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn streaming_normalization_bounds_batches_without_losing_captures() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("large-claude.jsonl");
        let mut file = File::create(&path).unwrap();
        for index in 0..40 {
            writeln!(
                file,
                "{}",
                json!({
                    "sessionId": "streaming-test",
                    "timestamp": format!("2026-07-20T12:00:{:02}Z", index % 60),
                    "cwd": "/workspace",
                    "type": if index % 2 == 0 { "user" } else { "assistant" },
                    "message": {
                        "role": if index % 2 == 0 { "user" } else { "assistant" },
                        "content": [{"type": "text", "text": format!("message {index}")}]
                    },
                    "uuid": format!("streaming-test-{index}")
                })
            )
            .unwrap();
        }
        drop(file);
        let path = path.canonicalize().unwrap();
        let context = ProviderAdapterContext::default();
        let normalized = normalize_claude_projects_jsonl_file(&path, &context, None).unwrap();
        let expected_captures = normalized.captures.len();
        let mut streamed_captures = 0usize;
        let mut batches = 0usize;
        let mut largest_batch = 0usize;

        import_large_claude_projects_jsonl_file_streaming(&path, &context, None, 512, |batch| {
            batches += 1;
            streamed_captures += batch.captures.len();
            largest_batch = largest_batch.max(batch.captures.len());
            Ok(ProviderImportSummary::default())
        })
        .unwrap();

        assert_eq!(streamed_captures, expected_captures);
        assert!(batches > 1, "expected multiple bounded batches");
        assert!(largest_batch < expected_captures);
    }
}

pub(crate) fn claude_path_session_ids(
    path: &Path,
    native_session_id: &str,
) -> (String, Option<String>, Option<String>, bool) {
    let Some(parent) = path.parent() else {
        return (native_session_id.to_owned(), None, None, false);
    };
    if parent.file_name().and_then(|name| name.to_str()) == Some("subagents") {
        let parent_session_id = parent
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(native_session_id)
            .to_owned();
        let agent_id = path
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("subagent")
            .to_owned();
        return (
            format!("{parent_session_id}/subagents/{agent_id}"),
            Some(parent_session_id),
            Some(agent_id),
            true,
        );
    }
    (native_session_id.to_owned(), None, None, false)
}

pub(crate) fn claude_event(
    value: &Value,
    line_number: usize,
    occurred_at: DateTime<Utc>,
) -> Option<ProviderEventEnvelope> {
    let entry_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let message = value.get("message").unwrap_or(value);
    let message_role = message
        .get("role")
        .and_then(Value::as_str)
        .or_else(|| value.get("role").and_then(Value::as_str));
    let null = Value::Null;
    let content = message.get("content").unwrap_or(&null);
    let event_type = claude_event_type(entry_type, message);
    let role = Some(provider_role(message_role));
    let text = provider_value_text(content).unwrap_or_else(|| {
        if event_type == EventType::Notice {
            format!("Claude event: {entry_type}")
        } else {
            String::new()
        }
    });
    let retained_text = provider_policy_event_text(event_type, &text, content);

    Some(ProviderEventEnvelope {
        provider_event_index: (line_number - 1) as u64,
        provider_event_hash: value.get("uuid").and_then(Value::as_str).map(str::to_owned),
        cursor: value.get("uuid").and_then(Value::as_str).map(str::to_owned),
        event_type,
        role,
        occurred_at,
        fidelity: Fidelity::Imported,
        idempotency_key: value
            .get("uuid")
            .and_then(Value::as_str)
            .map(|uuid| format!("provider-event:claude:{uuid}")),
        artifacts: Vec::new(),
        payload: json!({
            "entry_type": entry_type,
            "uuid": value.get("uuid").and_then(Value::as_str),
            "parent_uuid": value.get("parentUuid").and_then(Value::as_str),
            "message_id": message.get("id").and_then(Value::as_str),
            "request_id": value.get("requestId").and_then(Value::as_str),
            "role": message_role,
            "text": retained_text.text,
            "text_retention": retained_text.retention.as_json(),
            "content_preview": provider_capped_json(&provider_policy_body(event_type, content), PROVIDER_MAX_PREVIEW_CHARS),
        }),
        metadata: json!({
            "source": "claude_projects_jsonl",
            "source_format": CLAUDE_PROJECTS_SOURCE_FORMAT,
            "line": line_number,
            "entry_type": entry_type,
            "model": message.get("model").and_then(Value::as_str),
            "usage": message.get("usage").cloned(),
            "stop_reason": message.get("stop_reason").and_then(Value::as_str),
            "is_sidechain": value.get("isSidechain").and_then(Value::as_bool),
            "tool_use_result": value.get("toolUseResult").map(|value| provider_policy_body(EventType::ToolOutput, value)),
        }),
    })
}

pub(crate) fn claude_event_type(entry_type: &str, message: &Value) -> EventType {
    if claude_content_has_type(message.get("content"), "tool_result")
        || message.get("toolUseResult").is_some()
    {
        return EventType::ToolOutput;
    }
    if claude_content_has_type(message.get("content"), "tool_use") {
        return EventType::ToolCall;
    }
    match entry_type {
        "user" | "assistant" => EventType::Message,
        "system"
        | "progress"
        | "permission-mode"
        | "last-prompt"
        | "queue-operation"
        | "attachment"
        | "file-history-snapshot"
        | "ai-title" => EventType::Notice,
        _ => EventType::Notice,
    }
}

pub(crate) fn claude_content_has_type(content: Option<&Value>, expected: &str) -> bool {
    content
        .and_then(Value::as_array)
        .map(|blocks| {
            blocks
                .iter()
                .any(|block| block.get("type").and_then(Value::as_str) == Some(expected))
        })
        .unwrap_or(false)
}
