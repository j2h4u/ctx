use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use uuid::Uuid;
use work_record_capture::{
    import_codex_history_jsonl, import_codex_session_jsonl, import_codex_session_tree,
    import_pi_session_jsonl, stable_capture_uuid, CodexHistoryImportOptions,
    CodexSessionImportOptions, PiSessionImportOptions, ProviderImportSummary,
};
use work_record_core::{
    database_path, default_data_root, CaptureProvider, ContextCitation, ContextCitationType, Event,
    EventType, Session, WorkRecord,
};
use work_record_store::Store;

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Parser)]
#[command(name = "ctx", about = "Search local agent history")]
struct Cli {
    #[arg(long, env = "CTX_DATA_ROOT", global = true)]
    data_root: Option<PathBuf>,
    #[command(subcommand)]
    command: CommandRoot,
}

#[derive(Debug, Subcommand)]
enum CommandRoot {
    #[command(about = "Create local ctx storage and show next steps")]
    Setup(SetupArgs),
    #[command(about = "Show local ctx index status")]
    Status(JsonArgs),
    #[command(about = "List configured and discovered agent history sources")]
    Sources(JsonArgs),
    #[command(about = "Index provider history into local search")]
    Import(ImportArgs),
    #[command(about = "List indexed agent history items")]
    List(ListArgs),
    #[command(about = "Show one indexed agent history item")]
    Show(ShowArgs),
    #[command(about = "Search indexed agent history")]
    Search(SearchArgs),
    #[command(about = "Render deterministic cited context for an agent")]
    Context(ContextArgs),
    #[command(about = "Check local ctx health")]
    Doctor(JsonArgs),
    #[command(about = "Validate local ctx storage")]
    Validate(JsonArgs),
}

#[derive(Debug, Args)]
struct SetupArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args, Clone)]
struct JsonArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ImportArgs {
    #[arg(long, value_enum)]
    provider: Option<ProviderArg>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    all: bool,
    #[arg(long)]
    resume: bool,
    #[arg(long)]
    json: bool,
}

impl ImportArgs {
    fn resume_mode(&self) -> &'static str {
        if self.resume {
            "idempotent_rescan"
        } else {
            "normal_scan"
        }
    }
}

#[derive(Debug, Args)]
struct ListArgs {
    #[arg(long, default_value_t = 20)]
    limit: usize,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ShowArgs {
    id: Uuid,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct SearchArgs {
    query: Option<String>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
    #[arg(long)]
    provider: Option<ProviderArg>,
    #[arg(long)]
    repo: Option<String>,
    #[arg(long)]
    since: Option<String>,
    #[arg(long)]
    primary_only: bool,
    #[arg(long)]
    include_subagents: bool,
    #[arg(long)]
    event_type: Option<String>,
    #[arg(long)]
    file: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ContextArgs {
    query: String,
    #[arg(long, default_value_t = 10)]
    limit: usize,
    #[arg(long, default_value_t = work_record_search::DEFAULT_MAX_TOKENS)]
    max_tokens: u32,
    #[arg(long)]
    provider: Option<ProviderArg>,
    #[arg(long)]
    repo: Option<String>,
    #[arg(long)]
    since: Option<String>,
    #[arg(long)]
    primary_only: bool,
    #[arg(long)]
    include_subagents: bool,
    #[arg(long)]
    event_type: Option<String>,
    #[arg(long)]
    file: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ProviderArg {
    Codex,
    Pi,
}

impl ProviderArg {
    fn capture_provider(self) -> CaptureProvider {
        match self {
            Self::Codex => CaptureProvider::Codex,
            Self::Pi => CaptureProvider::Pi,
        }
    }

    fn as_str(self) -> &'static str {
        self.capture_provider().as_str()
    }
}

#[derive(Debug, Clone)]
struct SourceInfo {
    provider: ProviderArg,
    path: PathBuf,
    exists: bool,
    source_format: &'static str,
    status: &'static str,
}

#[derive(Debug, Default)]
struct ImportTotals {
    source_files: usize,
    source_bytes: u64,
    imported_sessions: usize,
    imported_events: usize,
    imported_edges: usize,
    skipped: usize,
    failed: usize,
}

impl ImportTotals {
    fn add(&mut self, summary: &ProviderImportSummary, stats: &SourceStats) {
        self.source_files += stats.files;
        self.source_bytes = self.source_bytes.saturating_add(stats.bytes);
        self.imported_sessions += summary.imported_sessions;
        self.imported_events += summary.imported_events;
        self.imported_edges += summary.imported_edges;
        self.skipped += summary.skipped;
        self.failed += summary.failed;
    }
}

#[derive(Debug, Default)]
struct SourceStats {
    files: usize,
    bytes: u64,
}

struct ListItemDto;
struct ShowDto;
struct SearchDto;
struct ContextDto;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let data_root = cli
        .data_root
        .clone()
        .map(Ok)
        .unwrap_or_else(default_data_root)
        .context("resolve ctx data root")?;

    match cli.command {
        CommandRoot::Setup(args) => run_setup(args, data_root),
        CommandRoot::Status(args) => run_status(args, data_root),
        CommandRoot::Sources(args) => run_sources(args),
        CommandRoot::Import(args) => run_import(args, data_root),
        CommandRoot::List(args) => run_list(args, data_root),
        CommandRoot::Show(args) => run_show(args, data_root),
        CommandRoot::Search(args) => run_search(args, data_root),
        CommandRoot::Context(args) => run_context(args, data_root),
        CommandRoot::Doctor(args) => run_doctor(args, data_root),
        CommandRoot::Validate(args) => run_validate(args, data_root),
    }
}

fn run_setup(args: SetupArgs, data_root: PathBuf) -> Result<()> {
    fs::create_dir_all(&data_root)?;
    let db_path = database_path(data_root.clone());
    let store = Store::open(&db_path)?;
    write_default_config(&data_root)?;
    let sources = discovered_sources();

    if args.json {
        print_json(json!({
            "schema_version": 1,
            "data_root": data_root,
            "database_path": store.path(),
            "config_path": data_root.join(CONFIG_FILE),
            "sources": sources_json(&sources),
            "network_required": false,
            "repo_writes": false,
        }))?;
    } else {
        println!("ctx local agent history search is ready");
        println!("data_root: {}", data_root.display());
        println!("database_path: {}", store.path().display());
        println!("config_path: {}", data_root.join(CONFIG_FILE).display());
        println!("next_steps:");
        println!("  ctx sources");
        println!("  ctx import --all");
        println!("  ctx search \"what failed before\"");
        println!("  ctx context \"what should the next agent know\"");
    }
    Ok(())
}

fn run_status(args: JsonArgs, data_root: PathBuf) -> Result<()> {
    let db_path = database_path(data_root.clone());
    let initialized = db_path.exists();
    let config_path = data_root.join(CONFIG_FILE);
    let (records, sources) = if initialized {
        let store = Store::open(&db_path)?;
        (
            store.list_records(usize::MAX)?.len() + store.list_sessions()?.len(),
            store.list_capture_sources()?.len(),
        )
    } else {
        (0, 0)
    };

    if args.json {
        print_json(json!({
            "schema_version": 1,
            "initialized": initialized,
            "data_root": data_root,
            "database_path": db_path,
            "config_path": config_path,
            "indexed_items": records,
            "indexed_sources": sources,
            "local_only": true,
        }))?;
    } else {
        println!("data_root: {}", data_root.display());
        println!("database_path: {}", db_path.display());
        println!("config_path: {}", config_path.display());
        println!("initialized: {initialized}");
        println!("indexed_items: {records}");
        println!("indexed_sources: {sources}");
        println!("local_only: true");
    }
    Ok(())
}

fn run_sources(args: JsonArgs) -> Result<()> {
    let sources = discovered_sources();
    if args.json {
        print_json(json!({
            "schema_version": 1,
            "sources": sources_json(&sources),
        }))?;
    } else {
        for source in sources {
            println!(
                "{} {} {} ({})",
                source.provider.as_str(),
                source.path.display(),
                source.status,
                source.source_format
            );
        }
    }
    Ok(())
}

fn run_import(args: ImportArgs, data_root: PathBuf) -> Result<()> {
    fs::create_dir_all(&data_root)?;
    write_default_config(&data_root)?;
    let mut store = Store::open(database_path(data_root))?;
    let mut totals = ImportTotals::default();
    let mut imported_sources = Vec::new();

    let requests = import_requests(&args)?;
    if requests.is_empty() {
        return Err(anyhow!(
            "no importable provider history sources found; use --path or run `ctx sources`"
        ));
    }

    for source in requests {
        let stats = source_stats(&source.path)
            .with_context(|| format!("scan import source {}", source.path.display()))?;
        if !args.json {
            println!(
                "importing {} {} ({} files, {} bytes)",
                source.provider.as_str(),
                source.path.display(),
                stats.files,
                stats.bytes
            );
        }
        let summary = import_one_source(&mut store, &source)?;
        totals.add(&summary, &stats);
        if !args.json {
            println!(
                "source_imported: sessions={} events={} edges={} skipped={} failed={}",
                summary.imported_sessions,
                summary.imported_events,
                summary.imported_edges,
                summary.skipped,
                summary.failed
            );
        }
        imported_sources.push(json!({
            "provider": source.provider.as_str(),
            "path": source.path,
            "source_format": source.source_format,
            "source_files": stats.files,
            "source_bytes": stats.bytes,
            "imported_sessions": summary.imported_sessions,
            "imported_events": summary.imported_events,
            "imported_edges": summary.imported_edges,
            "skipped": summary.skipped,
            "failed": summary.failed,
        }));
    }

    if args.json {
        print_json(json!({
            "schema_version": 1,
            "resume": args.resume,
            "resume_mode": args.resume_mode(),
            "totals": {
                "source_files": totals.source_files,
                "source_bytes": totals.source_bytes,
                "imported_sessions": totals.imported_sessions,
                "imported_events": totals.imported_events,
                "imported_edges": totals.imported_edges,
                "skipped": totals.skipped,
                "failed": totals.failed,
            },
            "sources": imported_sources,
        }))?;
    } else {
        println!("source_files: {}", totals.source_files);
        println!("source_bytes: {}", totals.source_bytes);
        println!("imported_sessions: {}", totals.imported_sessions);
        println!("imported_events: {}", totals.imported_events);
        println!("imported_edges: {}", totals.imported_edges);
        println!("skipped: {}", totals.skipped);
        println!("failed: {}", totals.failed);
        println!("resume: {}", args.resume);
        println!("resume_mode: {}", args.resume_mode());
    }
    Ok(())
}

fn run_list(args: ListArgs, data_root: PathBuf) -> Result<()> {
    let store = Store::open(database_path(data_root))?;
    let records = store.list_records(args.limit)?;
    let remaining = args.limit.saturating_sub(records.len());
    let sessions = store
        .list_sessions()?
        .into_iter()
        .take(remaining)
        .collect::<Vec<_>>();
    if args.json {
        let mut items = Vec::new();
        for record in records {
            items.push(ListItemDto::record(&record));
        }
        for session in sessions {
            items.push(ListItemDto::session(&session));
        }
        print_json(json!({
            "schema_version": 1,
            "items": items,
        }))?;
    } else {
        for record in records {
            println!("{} {}", record.id, record.title);
        }
        for session in sessions {
            println!(
                "{} session {}",
                session.id,
                session
                    .external_session_id
                    .unwrap_or_else(|| session.provider.to_string())
            );
        }
    }
    Ok(())
}

fn run_show(args: ShowArgs, data_root: PathBuf) -> Result<()> {
    let store = Store::open(database_path(data_root))?;
    let Ok(record) = store.get_record(args.id) else {
        let session = store.get_session(args.id)?;
        let events = store.events_for_session(session.id)?;
        if args.json {
            print_json(compact_json(json!({
                "schema_version": 1,
                "item": ShowDto::session(&store, &session),
                "events": events
                    .iter()
                    .map(|event| ShowDto::event(&store, event))
                    .collect::<Vec<_>>(),
            })))?;
        } else {
            println!("id: {}", session.id);
            println!("kind: session");
            println!("provider: {}", session.provider);
            if let Some(external_session_id) = session.external_session_id {
                println!("external_session_id: {external_session_id}");
            }
            if !events.is_empty() {
                println!();
                println!("events:");
                for event in events.iter().take(20) {
                    println!(
                        "  {} {:?} {}",
                        event.id,
                        event.event_type,
                        event_preview(event)
                    );
                }
            }
        }
        return Ok(());
    };
    let sessions = store.sessions_for_record(record.id)?;
    let events = store.events_for_record(record.id)?;
    if args.json {
        print_json(compact_json(json!({
            "schema_version": 1,
            "item": ShowDto::record(&record),
            "sessions": sessions
                .iter()
                .map(|session| ShowDto::session(&store, session))
                .collect::<Vec<_>>(),
            "events": events
                .iter()
                .map(|event| ShowDto::event(&store, event))
                .collect::<Vec<_>>(),
        })))?;
    } else {
        println!("id: {}", record.id);
        println!("title: {}", record.title);
        if !record.body.trim().is_empty() {
            println!();
            println!("{}", record.body);
        }
        if !sessions.is_empty() {
            println!();
            println!("sessions:");
            for session in sessions {
                println!(
                    "  {} {} {:?}",
                    session.id, session.provider, session.agent_type
                );
            }
        }
        if !events.is_empty() {
            println!();
            println!("events:");
            for event in events.iter().take(20) {
                println!("  {} {}", event.id, event.event_type.as_str());
            }
        }
    }
    Ok(())
}

fn event_preview(event: &Event) -> String {
    for key in ["text", "summary", "command", "message"] {
        if let Some(value) = event.payload.get(key).and_then(|value| value.as_str()) {
            return work_record_search::redacted_snippet(value, 120);
        }
    }
    if let Some(body) = event.payload.get("body") {
        for key in [
            "arguments_preview",
            "text",
            "summary",
            "command",
            "message",
            "tool",
            "name",
        ] {
            if let Some(value) = body.get(key).and_then(|value| value.as_str()) {
                return work_record_search::redacted_snippet(value, 120);
            }
        }
    }
    format!("{} event", event.event_type.as_str())
}

impl ListItemDto {
    fn record(record: &WorkRecord) -> Value {
        compact_json(json!({
            "id": record.id,
            "item_id": record.id,
            "item_type": public_record_item_type(record),
            "title": record.title,
            "created_at": record.created_at,
            "updated_at": record.updated_at,
        }))
    }

    fn session(session: &Session) -> Value {
        compact_json(json!({
            "id": session.id,
            "item_id": session.id,
            "item_type": "session",
            "provider": session.provider,
            "external_session_id": session.external_session_id,
            "agent_type": session.agent_type,
            "started_at": session.started_at,
            "ended_at": session.ended_at,
        }))
    }
}

impl ShowDto {
    fn record(record: &WorkRecord) -> Value {
        compact_json(json!({
            "id": record.id,
            "item_id": record.id,
            "item_type": public_record_item_type(record),
            "title": record.title,
            "text": record.body,
            "tags": record.tags,
            "workspace": record.workspace,
            "created_at": record.created_at,
            "updated_at": record.updated_at,
        }))
    }

    fn session(store: &Store, session: &Session) -> Value {
        let source_path = source_path_for(store, session.capture_source_id);
        compact_json(json!({
            "id": session.id,
            "item_id": session.id,
            "item_type": "session",
            "provider": session.provider,
            "external_session_id": session.external_session_id,
            "agent_type": session.agent_type,
            "role": session.role_hint,
            "is_primary": session.is_primary,
            "status": session.status,
            "started_at": session.started_at,
            "ended_at": session.ended_at,
            "source_id": session.capture_source_id,
            "source_path": source_path,
            "source_exists": source_path_exists(source_path.as_deref()),
        }))
    }

    fn event(store: &Store, event: &Event) -> Value {
        let source_path = source_path_for(store, event.capture_source_id);
        compact_json(json!({
            "event_id": event.id,
            "item_id": event.id,
            "item_type": "event",
            "session_id": event.session_id,
            "sequence": event.seq,
            "event_type": event.event_type,
            "role": event.role,
            "occurred_at": event.occurred_at,
            "source_id": event.capture_source_id,
            "source_path": source_path,
            "source_exists": source_path_exists(source_path.as_deref()),
            "cursor": event_cursor(event),
            "preview": event_preview(event),
            "redaction_state": event.redaction_state,
        }))
    }
}

impl SearchDto {
    fn packet(store: &Store, packet: &work_record_search::SearchPacket) -> Value {
        compact_json(json!({
            "schema_version": packet.schema_version,
            "query": packet.query,
            "filters": packet.filters,
            "generated_at": packet.generated_at,
            "results": packet
                .results
                .iter()
                .map(|result| {
                    compact_json(json!({
                        "item_id": result.record_id,
                        "item_type": item_type_for_id(store, result.record_id),
                        "session_id": result.session_id,
                        "event_id": result.event_id,
                        "event_seq": result.event_seq,
                        "title": result.title,
                        "snippet": result.snippet,
                        "rank": result.rank,
                        "provider": result.provider,
                        "timestamp": result.timestamp,
                        "cwd": result.cwd,
                        "source_path": result.raw_source_path,
                        "source_exists": result.raw_source_exists,
                        "cursor": result.cursor,
                        "why_matched": result.why_matched,
                        "citations": public_citations(&result.citations),
                        "links": result.links,
                        "visibility": result.visibility,
                    }))
                })
                .collect::<Vec<_>>(),
            "pagination": packet.pagination,
            "truncation": packet.truncation,
        }))
    }
}

impl ContextDto {
    fn packet(store: &Store, packet: &work_record_core::AgentContextPacket) -> Value {
        compact_json(json!({
            "schema_version": packet.schema_version,
            "query": packet.query,
            "filters": packet.filters,
            "generated_at": packet.generated_at,
            "budget": packet.budget,
            "results": packet
                .results
                .iter()
                .map(|result| {
                    compact_json(json!({
                        "item_id": result.record_id,
                        "item_type": item_type_for_id(store, result.record_id),
                        "title": result.title,
                        "summary": result.summary,
                        "rank": result.rank,
                        "why_matched": result.why_matched,
                        "citations": public_citations(&result.citations),
                        "links": result.links,
                        "visibility": result.visibility,
                    }))
                })
                .collect::<Vec<_>>(),
            "pagination": packet.pagination,
            "truncation": packet.truncation,
        }))
    }
}

fn public_citations(citations: &[ContextCitation]) -> Vec<Value> {
    citations
        .iter()
        .map(|citation| {
            compact_json(json!({
                "item_id": citation.id,
                "item_type": public_citation_item_type(citation.citation_type),
                "label": citation.label,
                "time": citation.time,
                "provider": citation.provider,
                "session_id": citation.session_id,
                "event_seq": citation.event_seq,
                "source_path": citation.raw_source_path,
                "source_exists": citation.raw_source_exists,
                "cursor": citation.cursor,
            }))
        })
        .collect()
}

fn public_citation_item_type(citation_type: ContextCitationType) -> &'static str {
    match citation_type {
        ContextCitationType::WorkRecord => "indexed_item",
        ContextCitationType::Session => "session",
        ContextCitationType::Run => "run",
        ContextCitationType::Event => "event",
        ContextCitationType::VcsChange => "vcs_change",
        ContextCitationType::Artifact => "artifact",
        ContextCitationType::Summary => "summary",
        ContextCitationType::File => "file",
    }
}

fn public_record_item_type(record: &WorkRecord) -> String {
    let item_type = record.kind.trim();
    match item_type {
        "" | "record" | "work_record" => "indexed_item".to_owned(),
        value => value.to_owned(),
    }
}

fn item_type_for_id(store: &Store, item_id: Uuid) -> String {
    store
        .get_record(item_id)
        .map(|record| public_record_item_type(&record))
        .unwrap_or_else(|_| "indexed_item".to_owned())
}

fn source_path_for(store: &Store, source_id: Option<Uuid>) -> Option<String> {
    source_id
        .and_then(|source_id| store.get_capture_source(source_id).ok())
        .and_then(|source| source.descriptor.raw_source_path)
}

fn source_path_exists(source_path: Option<&str>) -> Option<bool> {
    source_path.map(|path| Path::new(path).exists())
}

fn event_cursor(event: &Event) -> Option<String> {
    if let Some(cursor) = event.payload.get("cursor").and_then(|value| value.as_str()) {
        return Some(cursor.to_owned());
    }
    event
        .payload
        .get("body")
        .and_then(|body| body.get("cursor"))
        .and_then(|value| value.as_str())
        .map(str::to_owned)
}

fn compact_json(mut value: Value) -> Value {
    prune_null_json(&mut value);
    value
}

fn prune_null_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, nested| {
                prune_null_json(nested);
                !nested.is_null()
            });
        }
        Value::Array(items) => {
            for item in items {
                prune_null_json(item);
            }
        }
        _ => {}
    }
}

fn run_search(args: SearchArgs, data_root: PathBuf) -> Result<()> {
    let store = Store::open(database_path(data_root))?;
    let query = args.query.unwrap_or_default();
    let options = work_record_search::PacketOptions {
        limit: args.limit,
        filters: search_filters(
            args.provider,
            args.repo.clone(),
            args.since.clone(),
            args.primary_only,
            args.include_subagents,
            args.event_type.clone(),
            args.file.clone(),
        )?,
        ..work_record_search::PacketOptions::default()
    };
    if args.json {
        let packet = work_record_search::search_packet(&store, &query, &options)?;
        print_share_safe_value(SearchDto::packet(&store, &packet))?;
    } else {
        let packet = work_record_search::search_packet(&store, &query, &options)?;
        for result in packet.results {
            println!("{} {}", result.record_id, result.title);
            println!("  {}", result.snippet);
            for citation in result.citations.iter().take(2) {
                println!(
                    "  citation: {} {}",
                    citation.citation_type.as_str(),
                    citation.id
                );
            }
        }
    }
    Ok(())
}

fn run_context(args: ContextArgs, data_root: PathBuf) -> Result<()> {
    let store = Store::open(database_path(data_root))?;
    let options = work_record_search::PacketOptions {
        limit: args.limit,
        max_tokens: args.max_tokens,
        filters: search_filters(
            args.provider,
            args.repo.clone(),
            args.since.clone(),
            args.primary_only,
            args.include_subagents,
            args.event_type.clone(),
            args.file.clone(),
        )?,
        ..work_record_search::PacketOptions::default()
    };
    let packet = work_record_search::context_packet(&store, Some(&args.query), &options)?;
    if args.json {
        print_share_safe_value(ContextDto::packet(&store, &packet))?;
    } else {
        println!("# ctx Context");
        println!();
        println!("query: {}", args.query);
        println!("max_tokens: {}", packet.budget.max_tokens);
        println!("estimated_tokens: {}", packet.budget.estimated_tokens);
        println!();
        for result in packet.results {
            println!("## {}", result.title);
            println!("id: {}", result.record_id);
            println!("rank: {:.3}", result.rank);
            if !result.why_matched.is_empty() {
                println!("matched: {}", result.why_matched.join(", "));
            }
            if let Some(summary) = result.summary {
                println!();
                println!("{summary}");
            }
            if !result.citations.is_empty() {
                println!();
                println!("citations:");
                for citation in result.citations {
                    print!("  - {} {}", citation.citation_type.as_str(), citation.id);
                    if let Some(provider) = citation.provider {
                        print!(" provider={}", provider.as_str());
                    }
                    if let Some(session_id) = citation.session_id {
                        print!(" session={session_id}");
                    }
                    if let Some(event_seq) = citation.event_seq {
                        print!(" event_seq={event_seq}");
                    }
                    if let Some(raw_source_path) = citation.raw_source_path {
                        print!(" source={raw_source_path}");
                    }
                    if let Some(cursor) = citation.cursor {
                        print!(" cursor={cursor}");
                    }
                    println!();
                }
            }
            println!();
        }
        if let Some(truncation) = packet.truncation {
            if truncation.truncated {
                println!(
                    "truncation: {}",
                    truncation.reason.unwrap_or_else(|| "limit".to_owned())
                );
            }
        }
    }
    Ok(())
}

fn run_doctor(args: JsonArgs, data_root: PathBuf) -> Result<()> {
    let store = Store::open(database_path(data_root.clone()))?;
    let mut findings = store.validate()?;
    if !data_root.exists() {
        findings.push(format!("data root does not exist: {}", data_root.display()));
    }
    if args.json {
        print_json(json!({
            "schema_version": 1,
            "ok": findings.is_empty(),
            "findings": findings,
        }))?;
    } else if findings.is_empty() {
        println!("ok");
    } else {
        for finding in findings {
            println!("{finding}");
        }
    }
    Ok(())
}

fn run_validate(args: JsonArgs, data_root: PathBuf) -> Result<()> {
    let store = Store::open(database_path(data_root))?;
    let findings = store.validate()?;
    if args.json {
        print_json(json!({
            "schema_version": 1,
            "valid": findings.is_empty(),
            "findings": findings,
        }))?;
    } else if findings.is_empty() {
        println!("valid");
    } else {
        for finding in findings {
            println!("{finding}");
        }
    }
    Ok(())
}

fn import_requests(args: &ImportArgs) -> Result<Vec<SourceInfo>> {
    if let Some(path) = &args.path {
        let provider = args.provider.unwrap_or(ProviderArg::Codex);
        return Ok(vec![source_for_path(provider, path.clone())]);
    }
    if args.all || args.provider.is_none() {
        return Ok(discovered_sources()
            .into_iter()
            .filter(|source| source.exists)
            .collect());
    }
    let provider = args.provider.expect("checked provider");
    Ok(discovered_sources()
        .into_iter()
        .filter(|source| source.provider.as_str() == provider.as_str() && source.exists)
        .collect())
}

fn import_one_source(store: &mut Store, source: &SourceInfo) -> Result<ProviderImportSummary> {
    let record = import_record_for_source(source);
    let record_id = record.id;
    store.upsert_record(&record)?;
    let summary = match source.provider {
        ProviderArg::Codex => {
            if source.path.is_dir() {
                import_codex_session_tree(
                    &source.path,
                    store,
                    CodexSessionImportOptions {
                        source_path: Some(source.path.clone()),
                        work_record_id: Some(record_id),
                        allow_partial_failures: true,
                        ..CodexSessionImportOptions::default()
                    },
                )
                .map_err(anyhow::Error::from)
            } else if source
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "history.jsonl")
            {
                import_codex_history_jsonl(
                    &source.path,
                    store,
                    CodexHistoryImportOptions {
                        source_path: Some(source.path.clone()),
                        work_record_id: Some(record_id),
                        allow_partial_failures: true,
                        ..CodexHistoryImportOptions::default()
                    },
                )
                .map_err(anyhow::Error::from)
            } else {
                import_codex_session_jsonl(
                    &source.path,
                    store,
                    CodexSessionImportOptions {
                        source_path: Some(source.path.clone()),
                        work_record_id: Some(record_id),
                        allow_partial_failures: true,
                        ..CodexSessionImportOptions::default()
                    },
                )
                .map_err(anyhow::Error::from)
            }
        }
        ProviderArg::Pi => import_pi_session_jsonl(
            &source.path,
            store,
            PiSessionImportOptions {
                source_path: Some(source.path.clone()),
                work_record_id: Some(record_id),
                allow_partial_failures: true,
                ..PiSessionImportOptions::default()
            },
        )
        .map_err(anyhow::Error::from),
    }?;
    store.refresh_search_index()?;
    Ok(summary)
}

fn source_stats(path: &Path) -> Result<SourceStats> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_file() {
        return Ok(SourceStats {
            files: 1,
            bytes: metadata.len(),
        });
    }
    if !metadata.file_type().is_dir() {
        return Ok(SourceStats::default());
    }

    let mut stats = SourceStats::default();
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file() {
                let metadata = entry.metadata()?;
                stats.files += 1;
                stats.bytes = stats.bytes.saturating_add(metadata.len());
            }
        }
    }
    Ok(stats)
}

fn import_record_for_source(source: &SourceInfo) -> WorkRecord {
    let key = format!(
        "agent-history:{}:{}",
        source.provider.as_str(),
        source.path.display()
    );
    let mut record = WorkRecord::new(
        format!("{} agent history", source.provider.as_str()),
        format!(
            "Indexed local agent history from {} ({})",
            source.path.display(),
            source.source_format
        ),
        vec!["agent-history".into(), source.provider.as_str().into()],
        "agent_history",
        source.path.parent().map(|path| path.display().to_string()),
    );
    record.id = stable_capture_uuid(&key, "record");
    record
}

fn discovered_sources() -> Vec<SourceInfo> {
    let mut sources = Vec::new();
    if let Some(home) = home_dir() {
        sources.push(source_for_path(
            ProviderArg::Codex,
            home.join(".codex").join("sessions"),
        ));
        sources.push(SourceInfo {
            provider: ProviderArg::Codex,
            path: home.join(".codex").join("history.jsonl"),
            exists: home.join(".codex").join("history.jsonl").exists(),
            source_format: "codex_history_jsonl",
            status: if home.join(".codex").join("history.jsonl").exists() {
                "available"
            } else {
                "missing"
            },
        });
        sources.push(source_for_path(
            ProviderArg::Pi,
            home.join(".pi").join("sessions.jsonl"),
        ));
    }
    sources
}

fn source_for_path(provider: ProviderArg, path: PathBuf) -> SourceInfo {
    let exists = path.exists();
    let source_format = match provider {
        ProviderArg::Codex if path.is_dir() => "codex_session_jsonl_tree",
        ProviderArg::Codex => "codex_session_jsonl",
        ProviderArg::Pi => "pi_session_jsonl",
    };
    SourceInfo {
        provider,
        path,
        exists,
        source_format,
        status: if exists { "available" } else { "missing" },
    }
}

fn sources_json(sources: &[SourceInfo]) -> Vec<Value> {
    sources
        .iter()
        .map(|source| {
            json!({
                "provider": source.provider.as_str(),
                "path": source.path,
                "exists": source.exists,
                "source_format": source.source_format,
                "status": source.status,
                "raw_retention": "path_reference",
            })
        })
        .collect()
}

fn search_filters(
    provider: Option<ProviderArg>,
    repo: Option<String>,
    since: Option<String>,
    primary_only: bool,
    include_subagents: bool,
    event_type: Option<String>,
    file: Option<PathBuf>,
) -> Result<work_record_search::SearchFilters> {
    Ok(work_record_search::SearchFilters {
        provider: provider.map(ProviderArg::capture_provider),
        repo,
        since: since.as_deref().map(parse_since_filter).transpose()?,
        primary_only,
        include_subagents: include_subagents || !primary_only,
        event_type: event_type
            .as_deref()
            .map(EventType::from_str)
            .transpose()
            .map_err(|err| anyhow!("{err}"))?,
        file: file.map(|path| path.display().to_string()),
    })
}

fn parse_since_filter(value: &str) -> Result<chrono::DateTime<Utc>> {
    let trimmed = value.trim();
    if let Some(days) = trimmed.strip_suffix('d') {
        let days: i64 = days
            .parse()
            .with_context(|| format!("invalid --since day window: {value}"))?;
        return Ok(Utc::now() - Duration::days(days));
    }
    Ok(chrono::DateTime::parse_from_rfc3339(trimmed)
        .with_context(|| format!("invalid --since value: {value}"))?
        .with_timezone(&Utc))
}

fn write_default_config(data_root: &Path) -> Result<()> {
    let path = data_root.join(CONFIG_FILE);
    if path.exists() {
        return Ok(());
    }
    let mut file = fs::File::create(&path)?;
    file.write_all(
        b"# ctx local agent history search\n\
data_root_version = 1\n\
network_during_import_search_context = false\n",
    )?;
    Ok(())
}

fn print_json(value: Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn print_share_safe_value(mut value: Value) -> Result<()> {
    mark_share_safe(&mut value);
    print_json(value)
}

fn mark_share_safe(value: &mut Value) {
    if let Value::Object(map) = value {
        map.entry("share_safe").or_insert(Value::Bool(false));
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
