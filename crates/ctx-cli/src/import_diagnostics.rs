use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use ctx_history_capture::{ProviderImportProgress, ProviderImportStage};
use ctx_history_core::utc_now;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{DiagnosticsArgs, DiagnosticsCommand};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const RETAIN_IMPORT_RUNS: usize = 50;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ImportSourceSnapshot {
    provider: String,
    path: String,
    stage: &'static str,
    completed_files: usize,
    total_files: usize,
    completed_bytes: u64,
    total_bytes: u64,
    completed_units: usize,
    total_units: usize,
    imported_sessions: usize,
    imported_events: usize,
    imported_edges: usize,
    failed: usize,
}

#[derive(Debug, Default, Serialize)]
struct ImportStageSummary {
    phase: String,
    elapsed_ms: u64,
    cpu_user_ms: u64,
    cpu_system_ms: u64,
    max_rss_bytes: u64,
    database_bytes_start: u64,
    database_bytes_end: u64,
    wal_bytes_max: u64,
    completed_bytes_start: u64,
    completed_bytes_end: u64,
    completed_units_start: u64,
    completed_units_end: u64,
}

#[derive(Debug)]
struct ImportDiagnosticState {
    phase: String,
    message: String,
    finished: bool,
    sources: BTreeMap<String, ImportSourceSnapshot>,
}

struct ImportDiagnosticsInner {
    run_id: Uuid,
    started: Instant,
    db_path: PathBuf,
    state: Mutex<ImportDiagnosticState>,
    writer: Mutex<BufWriter<File>>,
}

#[derive(Clone)]
pub(crate) struct ImportDiagnostics {
    inner: Arc<ImportDiagnosticsInner>,
}

pub(crate) struct ImportDiagnosticsGuard {
    diagnostics: ImportDiagnostics,
    completed: bool,
}

impl ImportDiagnostics {
    pub(crate) fn start(
        data_root: &Path,
        db_path: &Path,
        operation: &str,
    ) -> Result<(ImportDiagnosticsGuard, ImportDiagnostics)> {
        let directory = data_root.join("diagnostics").join("import-runs");
        fs::create_dir_all(&directory).with_context(|| {
            format!(
                "create import diagnostics directory {}",
                directory.display()
            )
        })?;
        prune_old_journals(&directory, RETAIN_IMPORT_RUNS.saturating_sub(1));
        let run_id = Uuid::now_v7();
        let timestamp = utc_now().format("%Y%m%dT%H%M%S%.3fZ");
        let journal_path = directory.join(format!("{timestamp}-{run_id}.jsonl"));
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&journal_path)
            .with_context(|| format!("create import journal {}", journal_path.display()))?;
        let diagnostics = Self {
            inner: Arc::new(ImportDiagnosticsInner {
                run_id,
                started: Instant::now(),
                db_path: db_path.to_path_buf(),
                state: Mutex::new(ImportDiagnosticState {
                    phase: "starting".to_owned(),
                    message: String::new(),
                    finished: false,
                    sources: BTreeMap::new(),
                }),
                writer: Mutex::new(BufWriter::new(file)),
            }),
        };
        diagnostics.write_record(
            json!({
                "type": "run_start",
                "schema_version": 1,
                "run_id": run_id,
                "timestamp": utc_now(),
                "pid": std::process::id(),
                "operation": operation,
                "database_path": db_path,
            }),
            true,
        );
        start_sampler(Arc::downgrade(&diagnostics.inner));
        let guard = ImportDiagnosticsGuard {
            diagnostics: diagnostics.clone(),
            completed: false,
        };
        Ok((guard, diagnostics))
    }

    pub(crate) fn phase(&self, phase: &str, message: impl Into<String>) {
        let message = message.into();
        {
            let mut state = self
                .inner
                .state
                .lock()
                .expect("import diagnostics poisoned");
            state.phase = phase.to_owned();
            state.message = message.clone();
        }
        self.write_record(
            json!({
                "type": "phase",
                "schema_version": 1,
                "run_id": self.inner.run_id,
                "timestamp": utc_now(),
                "elapsed_ms": self.inner.started.elapsed().as_millis(),
                "phase": phase,
                "message": message,
            }),
            true,
        );
    }

    pub(crate) fn provider_progress(&self, provider: &str, progress: &ProviderImportProgress) {
        let path = progress
            .source_path
            .as_deref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| provider.to_owned());
        let key = format!("{provider}\0{path}");
        let stage = match progress.stage {
            ProviderImportStage::Reading => "read",
            ProviderImportStage::Writing => "write",
            ProviderImportStage::Searching => "search",
        };
        let phase = match progress.stage {
            ProviderImportStage::Reading | ProviderImportStage::Writing => "import_ingest",
            ProviderImportStage::Searching => "import_search",
        }
        .to_owned();
        let phase_changed = {
            let mut state = self
                .inner
                .state
                .lock()
                .expect("import diagnostics poisoned");
            let changed = state.phase != phase;
            state.phase = phase.clone();
            state.message = format!("{provider} {stage}");
            state.sources.insert(
                key,
                ImportSourceSnapshot {
                    provider: provider.to_owned(),
                    path,
                    stage,
                    completed_files: progress.completed_files,
                    total_files: progress.total_files,
                    completed_bytes: progress.completed_bytes,
                    total_bytes: progress.total_bytes,
                    completed_units: progress.completed_units,
                    total_units: progress.total_units,
                    imported_sessions: progress.imported_sessions,
                    imported_events: progress.imported_events,
                    imported_edges: progress.imported_edges,
                    failed: progress.failed,
                },
            );
            changed
        };
        if phase_changed {
            self.write_record(
                json!({
                    "type": "phase",
                    "schema_version": 1,
                    "run_id": self.inner.run_id,
                    "timestamp": utc_now(),
                    "elapsed_ms": self.inner.started.elapsed().as_millis(),
                    "phase": phase,
                    "message": format!("{provider} {stage}"),
                }),
                true,
            );
        }
    }

    fn finish(&self, status: &str) {
        {
            let mut state = self
                .inner
                .state
                .lock()
                .expect("import diagnostics poisoned");
            if state.finished {
                return;
            }
            state.finished = true;
        }
        write_sample(&self.inner);
        self.write_record(
            json!({
                "type": "run_end",
                "schema_version": 1,
                "run_id": self.inner.run_id,
                "timestamp": utc_now(),
                "elapsed_ms": self.inner.started.elapsed().as_millis(),
                "status": status,
            }),
            true,
        );
    }

    fn write_record(&self, value: serde_json::Value, sync: bool) {
        write_json_line(&self.inner, value, sync);
    }
}

impl ImportDiagnosticsGuard {
    pub(crate) fn complete(mut self) {
        self.diagnostics.finish("complete");
        self.completed = true;
    }
}

impl Drop for ImportDiagnosticsGuard {
    fn drop(&mut self) {
        if !self.completed && !std::thread::panicking() {
            self.diagnostics.finish("failed");
        }
    }
}

fn start_sampler(inner: Weak<ImportDiagnosticsInner>) {
    let _ = thread::Builder::new()
        .name("ctx-import-metrics".to_owned())
        .spawn(move || loop {
            thread::sleep(SAMPLE_INTERVAL);
            let Some(inner) = inner.upgrade() else {
                break;
            };
            if inner
                .state
                .lock()
                .expect("import diagnostics poisoned")
                .finished
            {
                break;
            }
            write_sample(&inner);
        });
}

fn write_sample(inner: &ImportDiagnosticsInner) {
    let (phase, message, sources) = {
        let state = inner.state.lock().expect("import diagnostics poisoned");
        (
            state.phase.clone(),
            state.message.clone(),
            state.sources.values().cloned().collect::<Vec<_>>(),
        )
    };
    let process = process_metrics();
    let db_bytes = file_len(&inner.db_path);
    let wal_bytes = file_len(&PathBuf::from(format!("{}-wal", inner.db_path.display())));
    write_json_line(
        inner,
        json!({
            "type": "sample",
            "schema_version": 1,
            "run_id": inner.run_id,
            "timestamp": utc_now(),
            "elapsed_ms": inner.started.elapsed().as_millis(),
            "phase": phase,
            "message": message,
            "process": process,
            "storage": { "database_bytes": db_bytes, "wal_bytes": wal_bytes },
            "sources": sources,
        }),
        false,
    );
}

fn write_json_line(inner: &ImportDiagnosticsInner, value: serde_json::Value, sync: bool) {
    let Ok(mut writer) = inner.writer.lock() else {
        return;
    };
    if serde_json::to_writer(&mut *writer, &value).is_err()
        || writer.write_all(b"\n").is_err()
        || writer.flush().is_err()
    {
        return;
    }
    if sync {
        let _ = writer.get_ref().sync_data();
    }
}

fn file_len(path: &Path) -> u64 {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}

fn prune_old_journals(directory: &Path, retain: usize) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut journals = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("jsonl"))
        .collect::<Vec<_>>();
    journals.sort();
    let remove_count = journals.len().saturating_sub(retain);
    for path in journals.into_iter().take(remove_count) {
        let _ = fs::remove_file(path);
    }
}

#[cfg(unix)]
fn process_metrics() -> serde_json::Value {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    // SAFETY: getrusage initializes the provided rusage structure on success.
    let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
    if result != 0 {
        return json!({});
    }
    // SAFETY: the successful getrusage call initialized usage.
    let usage = unsafe { usage.assume_init() };
    let timeval_ms = |value: libc::timeval| {
        (value.tv_sec as i128 * 1_000 + value.tv_usec as i128 / 1_000).max(0) as u64
    };
    #[cfg(target_os = "macos")]
    let rss_bytes = usage.ru_maxrss.max(0) as u64;
    #[cfg(not(target_os = "macos"))]
    let rss_bytes = (usage.ru_maxrss.max(0) as u64).saturating_mul(1024);
    json!({
        "cpu_user_ms": timeval_ms(usage.ru_utime),
        "cpu_system_ms": timeval_ms(usage.ru_stime),
        "max_rss_bytes": rss_bytes,
        "minor_faults": usage.ru_minflt.max(0) as u64,
        "major_faults": usage.ru_majflt.max(0) as u64,
    })
}

#[cfg(not(unix))]
fn process_metrics() -> serde_json::Value {
    json!({})
}

pub(crate) fn run(args: DiagnosticsArgs, data_root: PathBuf) -> Result<()> {
    match args.command {
        DiagnosticsCommand::Imports(args) => show_import_run(
            &data_root,
            args.run.as_deref(),
            args.compare.as_deref(),
            args.json,
        ),
    }
}

fn show_import_run(
    data_root: &Path,
    requested: Option<&str>,
    compare: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let directory = data_root.join("diagnostics").join("import-runs");
    let primary_path = find_journal(&directory, requested)?;
    let primary = read_journal_summary(&primary_path)?;
    let comparison = compare
        .map(|run| find_journal(&directory, Some(run)))
        .transpose()?
        .map(|path| read_journal_summary(&path))
        .transpose()?;
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "run": primary,
                "compare": comparison,
            }))?
        );
        return Ok(());
    }
    print_summary(&primary);
    if let Some(comparison) = comparison {
        println!("\ncomparison:");
        print_summary(&comparison);
        let primary_ms = primary["elapsed_ms"].as_u64().unwrap_or(0);
        let comparison_ms = comparison["elapsed_ms"].as_u64().unwrap_or(0);
        if comparison_ms > 0 {
            println!(
                "elapsed ratio: {:.2}x",
                primary_ms as f64 / comparison_ms as f64
            );
        }
    }
    Ok(())
}

fn find_journal(directory: &Path, requested: Option<&str>) -> Result<PathBuf> {
    let mut journals = fs::read_dir(directory)
        .with_context(|| format!("read import diagnostics directory {}", directory.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("jsonl"))
        .collect::<Vec<_>>();
    journals.sort();
    if let Some(requested) = requested {
        return journals
            .into_iter()
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains(requested))
            })
            .with_context(|| format!("import diagnostic run {requested} not found"));
    }
    journals
        .pop()
        .context("no import diagnostic runs found; run `ctx setup` or `ctx import` first")
}

fn read_journal_summary(path: &Path) -> Result<serde_json::Value> {
    let file = File::open(path)
        .with_context(|| format!("open import diagnostic journal {}", path.display()))?;
    let mut start = None;
    let mut last_sample = None;
    let mut end = None;
    let mut samples = Vec::new();
    let mut valid_records = 0usize;
    for line in BufReader::new(file).lines() {
        let Ok(line) = line else { continue };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        valid_records = valid_records.saturating_add(1);
        match value["type"].as_str() {
            Some("run_start") => start = Some(value),
            Some("sample") => {
                last_sample = Some(value.clone());
                samples.push(value);
            }
            Some("run_end") => end = Some(value),
            _ => {}
        }
    }
    let start = start.context("diagnostic journal has no valid run_start record")?;
    let sample = last_sample.unwrap_or_else(|| json!({}));
    let elapsed_ms = end
        .as_ref()
        .and_then(|value| value["elapsed_ms"].as_u64())
        .or_else(|| sample["elapsed_ms"].as_u64())
        .unwrap_or(0);
    let stages = summarize_stages(&samples, elapsed_ms);
    Ok(json!({
        "run_id": start["run_id"],
        "started_at": start["timestamp"],
        "status": end.as_ref().and_then(|value| value["status"].as_str()).unwrap_or("interrupted"),
        "elapsed_ms": elapsed_ms,
        "phase": sample["phase"],
        "message": sample["message"],
        "process": sample["process"],
        "storage": sample["storage"],
        "sources": sample["sources"],
        "stages": stages,
        "valid_records": valid_records,
        "journal_path": path,
    }))
}

fn summarize_stages(samples: &[serde_json::Value], run_elapsed_ms: u64) -> Vec<ImportStageSummary> {
    let mut stages = Vec::<ImportStageSummary>::new();
    let mut previous_elapsed = 0u64;
    let mut previous_user = 0u64;
    let mut previous_system = 0u64;

    for sample in samples {
        let phase = sample["phase"].as_str().unwrap_or("unknown");
        let elapsed = sample["elapsed_ms"].as_u64().unwrap_or(previous_elapsed);
        let user = sample["process"]["cpu_user_ms"]
            .as_u64()
            .unwrap_or(previous_user);
        let system = sample["process"]["cpu_system_ms"]
            .as_u64()
            .unwrap_or(previous_system);
        let database = sample["storage"]["database_bytes"].as_u64().unwrap_or(0);
        let wal = sample["storage"]["wal_bytes"].as_u64().unwrap_or(0);
        let (bytes, units) = sample_totals_for_phase(sample, phase);
        let stage = match stages.iter_mut().find(|stage| stage.phase == phase) {
            Some(stage) => stage,
            None => {
                stages.push(ImportStageSummary {
                    phase: phase.to_owned(),
                    database_bytes_start: database,
                    completed_bytes_start: bytes,
                    completed_units_start: units,
                    ..ImportStageSummary::default()
                });
                stages.last_mut().expect("stage was just inserted")
            }
        };
        stage.elapsed_ms = stage
            .elapsed_ms
            .saturating_add(elapsed.saturating_sub(previous_elapsed));
        stage.cpu_user_ms = stage
            .cpu_user_ms
            .saturating_add(user.saturating_sub(previous_user));
        stage.cpu_system_ms = stage
            .cpu_system_ms
            .saturating_add(system.saturating_sub(previous_system));
        stage.max_rss_bytes = stage
            .max_rss_bytes
            .max(sample["process"]["max_rss_bytes"].as_u64().unwrap_or(0));
        stage.database_bytes_end = database;
        stage.wal_bytes_max = stage.wal_bytes_max.max(wal);
        stage.completed_bytes_end = bytes;
        stage.completed_units_end = units;
        previous_elapsed = elapsed;
        previous_user = user;
        previous_system = system;
    }
    if let Some(stage) = stages.last_mut() {
        stage.elapsed_ms = stage
            .elapsed_ms
            .saturating_add(run_elapsed_ms.saturating_sub(previous_elapsed));
    }
    stages
}

fn sample_totals_for_phase(sample: &serde_json::Value, phase: &str) -> (u64, u64) {
    let expected_stage = phase.strip_prefix("import_");
    sample["sources"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|source| expected_stage.is_none_or(|stage| source["stage"].as_str() == Some(stage)))
        .fold((0u64, 0u64), |(bytes, units), source| {
            (
                bytes.saturating_add(source["completed_bytes"].as_u64().unwrap_or(0)),
                units.saturating_add(source["completed_units"].as_u64().unwrap_or(0)),
            )
        })
}

fn print_summary(summary: &serde_json::Value) {
    let elapsed = summary["elapsed_ms"].as_u64().unwrap_or(0) as f64 / 1_000.0;
    println!("run: {}", summary["run_id"].as_str().unwrap_or("unknown"));
    println!(
        "status: {}",
        summary["status"].as_str().unwrap_or("unknown")
    );
    println!("elapsed: {elapsed:.1}s");
    println!("phase: {}", summary["phase"].as_str().unwrap_or("unknown"));
    let process = &summary["process"];
    println!(
        "cpu: user {:.1}s + system {:.1}s · max RSS {}",
        process["cpu_user_ms"].as_u64().unwrap_or(0) as f64 / 1_000.0,
        process["cpu_system_ms"].as_u64().unwrap_or(0) as f64 / 1_000.0,
        crate::progress::format_bytes(process["max_rss_bytes"].as_u64().unwrap_or(0)),
    );
    println!(
        "storage: DB {} · WAL {}",
        crate::progress::format_bytes(summary["storage"]["database_bytes"].as_u64().unwrap_or(0)),
        crate::progress::format_bytes(summary["storage"]["wal_bytes"].as_u64().unwrap_or(0)),
    );
    if let Some(stages) = summary["stages"].as_array() {
        println!("stages:");
        for stage in stages {
            println!(
                "  {:<18} {:>6.1}s · CPU {:>5.1}s · RSS {:>8} · DB +{} · WAL max {}",
                stage["phase"].as_str().unwrap_or("unknown"),
                stage["elapsed_ms"].as_u64().unwrap_or(0) as f64 / 1_000.0,
                (stage["cpu_user_ms"].as_u64().unwrap_or(0)
                    + stage["cpu_system_ms"].as_u64().unwrap_or(0)) as f64
                    / 1_000.0,
                crate::progress::format_bytes(stage["max_rss_bytes"].as_u64().unwrap_or(0)),
                crate::progress::format_bytes(
                    stage["database_bytes_end"]
                        .as_u64()
                        .unwrap_or(0)
                        .saturating_sub(stage["database_bytes_start"].as_u64().unwrap_or(0))
                ),
                crate::progress::format_bytes(stage["wal_bytes_max"].as_u64().unwrap_or(0)),
            );
        }
    }
    if let Some(sources) = summary["sources"].as_array() {
        for source in sources {
            println!(
                "{} {} · {} · {}/{} files · {}/{} units",
                source["provider"].as_str().unwrap_or("source"),
                source["path"].as_str().unwrap_or(""),
                source["stage"].as_str().unwrap_or("unknown"),
                source["completed_files"].as_u64().unwrap_or(0),
                source["total_files"].as_u64().unwrap_or(0),
                source["completed_units"].as_u64().unwrap_or(0),
                source["total_units"].as_u64().unwrap_or(0),
            );
        }
    }
    println!(
        "journal: {}",
        summary["journal_path"].as_str().unwrap_or("")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_run_keeps_readable_stage_metrics() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("work.sqlite");
        fs::write(&db_path, b"db").unwrap();
        let (guard, diagnostics) =
            ImportDiagnostics::start(temp.path(), &db_path, "setup").unwrap();
        diagnostics.phase("indexing", "Writing Claude history");
        diagnostics.provider_progress(
            "claude",
            &ProviderImportProgress {
                stage: ProviderImportStage::Writing,
                source_path: Some(PathBuf::from("/tmp/.claude-work/projects")),
                total_files: 10,
                total_bytes: 1_024,
                completed_files: 4,
                completed_bytes: 512,
                completed_units: 80,
                total_units: 200,
                imported_sessions: 3,
                imported_events: 70,
                imported_edges: 2,
                skipped: 0,
                failed: 0,
                done: false,
            },
        );
        write_sample(&diagnostics.inner);
        guard.complete();

        let journal =
            find_journal(&temp.path().join("diagnostics").join("import-runs"), None).unwrap();
        fs::OpenOptions::new()
            .append(true)
            .open(&journal)
            .unwrap()
            .write_all(b"{truncated")
            .unwrap();
        let summary = read_journal_summary(&journal).unwrap();

        assert_eq!(summary["status"], "complete");
        assert_eq!(summary["phase"], "import_ingest");
        assert_eq!(summary["sources"][0]["stage"], "write");
        assert_eq!(summary["sources"][0]["completed_units"], 80);
        assert_eq!(summary["storage"]["database_bytes"], 2);
    }
}
