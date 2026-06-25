use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    thread,
    time::Instant,
};

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Count,
    ValueAll,
    ValueNoOutput,
    OutputMeta,
}

impl Mode {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "count" => Ok(Self::Count),
            "value-all" => Ok(Self::ValueAll),
            "value-no-output" => Ok(Self::ValueNoOutput),
            "output-meta" => Ok(Self::OutputMeta),
            _ => Err(anyhow!(
                "unknown mode {value}; expected count, value-all, value-no-output, or output-meta"
            )),
        }
    }
}

#[derive(Debug, Clone)]
struct Options {
    root: PathBuf,
    mode: Mode,
    workers: usize,
    max_files: Option<usize>,
    max_bytes: Option<u64>,
}

#[derive(Debug, Default, Clone)]
struct Stats {
    files: usize,
    bytes: u64,
    lines: usize,
    relevant_lines: usize,
    parsed_lines: usize,
    failed_lines: usize,
    session_meta: usize,
    messages: usize,
    user_messages: usize,
    assistant_messages: usize,
    message_chars: usize,
    reasoning: usize,
    tool_calls: usize,
    tool_call_arg_chars: usize,
    tool_outputs: usize,
    tool_output_lines: usize,
    tool_output_bytes: u64,
    tool_output_preview_chars: usize,
    command_outputs: usize,
    exit_codes: usize,
    wall_times: usize,
    compacted: usize,
    event_msgs: usize,
}

impl Stats {
    fn merge(&mut self, other: Stats) {
        self.files += other.files;
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.lines += other.lines;
        self.relevant_lines += other.relevant_lines;
        self.parsed_lines += other.parsed_lines;
        self.failed_lines += other.failed_lines;
        self.session_meta += other.session_meta;
        self.messages += other.messages;
        self.user_messages += other.user_messages;
        self.assistant_messages += other.assistant_messages;
        self.message_chars += other.message_chars;
        self.reasoning += other.reasoning;
        self.tool_calls += other.tool_calls;
        self.tool_call_arg_chars += other.tool_call_arg_chars;
        self.tool_outputs += other.tool_outputs;
        self.tool_output_lines += other.tool_output_lines;
        self.tool_output_bytes = self
            .tool_output_bytes
            .saturating_add(other.tool_output_bytes);
        self.tool_output_preview_chars += other.tool_output_preview_chars;
        self.command_outputs += other.command_outputs;
        self.exit_codes += other.exit_codes;
        self.wall_times += other.wall_times;
        self.compacted += other.compacted;
        self.event_msgs += other.event_msgs;
    }
}

fn main() -> Result<()> {
    let options = parse_args()?;
    let started = Instant::now();
    let mut paths = Vec::new();
    collect_jsonl_paths(&options.root, &mut paths)?;
    paths.sort();
    apply_bounds(&mut paths, options.max_files, options.max_bytes)?;
    let discovered = paths.len();
    let stats = run(paths, options.mode, options.workers)?;
    let elapsed = started.elapsed();
    println!(
        "{{\"mode\":\"{:?}\",\"workers\":{},\"discovered_files\":{},\"elapsed_ms\":{},\"files\":{},\"bytes\":{},\"lines\":{},\"relevant_lines\":{},\"parsed_lines\":{},\"failed_lines\":{},\"session_meta\":{},\"messages\":{},\"user_messages\":{},\"assistant_messages\":{},\"message_chars\":{},\"reasoning\":{},\"tool_calls\":{},\"tool_call_arg_chars\":{},\"tool_outputs\":{},\"tool_output_lines\":{},\"tool_output_bytes\":{},\"tool_output_preview_chars\":{},\"command_outputs\":{},\"exit_codes\":{},\"wall_times\":{},\"compacted\":{},\"event_msgs\":{}}}",
        options.mode,
        options.workers,
        discovered,
        elapsed.as_millis(),
        stats.files,
        stats.bytes,
        stats.lines,
        stats.relevant_lines,
        stats.parsed_lines,
        stats.failed_lines,
        stats.session_meta,
        stats.messages,
        stats.user_messages,
        stats.assistant_messages,
        stats.message_chars,
        stats.reasoning,
        stats.tool_calls,
        stats.tool_call_arg_chars,
        stats.tool_outputs,
        stats.tool_output_lines,
        stats.tool_output_bytes,
        stats.tool_output_preview_chars,
        stats.command_outputs,
        stats.exit_codes,
        stats.wall_times,
        stats.compacted,
        stats.event_msgs
    );
    Ok(())
}

fn parse_args() -> Result<Options> {
    let mut args = env::args().skip(1);
    let mut root = None;
    let mut mode = Mode::ValueAll;
    let mut workers = thread::available_parallelism()
        .ok()
        .map(usize::from)
        .unwrap_or(1);
    let mut max_files = None;
    let mut max_bytes = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => {
                root = Some(PathBuf::from(
                    args.next().context("--root requires a path")?,
                ))
            }
            "--mode" => mode = Mode::parse(&args.next().context("--mode requires a value")?)?,
            "--workers" => {
                workers = args
                    .next()
                    .context("--workers requires a value")?
                    .parse()
                    .context("parse --workers")?
            }
            "--max-files" => {
                max_files = Some(
                    args.next()
                        .context("--max-files requires a value")?
                        .parse()
                        .context("parse --max-files")?,
                )
            }
            "--max-bytes" => {
                max_bytes = Some(
                    args.next()
                        .context("--max-bytes requires a value")?
                        .parse()
                        .context("parse --max-bytes")?,
                )
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => return Err(anyhow!("unknown argument {arg}")),
        }
    }
    Ok(Options {
        root: root.unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".codex")
                .join("sessions")
        }),
        mode,
        workers: workers.max(1),
        max_files,
        max_bytes,
    })
}

fn print_help() {
    println!("usage: codex_parse_bench [--root PATH] [--mode MODE] [--workers N] [--max-files N] [--max-bytes N]");
    println!("modes: count, value-all, value-no-output, output-meta");
}

fn run(paths: Vec<PathBuf>, mode: Mode, workers: usize) -> Result<Stats> {
    if paths.is_empty() {
        return Ok(Stats::default());
    }
    let worker_count = workers.min(paths.len());
    if worker_count == 1 {
        return parse_chunk(paths, mode);
    }
    let chunk_size = paths.len().div_ceil(worker_count).max(1);
    thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in paths.chunks(chunk_size) {
            let chunk = chunk.to_vec();
            handles.push(scope.spawn(move || parse_chunk(chunk, mode)));
        }
        let mut merged = Stats::default();
        for handle in handles {
            merged.merge(handle.join().map_err(|_| anyhow!("worker panicked"))??);
        }
        Ok(merged)
    })
}

fn parse_chunk(paths: Vec<PathBuf>, mode: Mode) -> Result<Stats> {
    let mut stats = Stats::default();
    for path in paths {
        let metadata = fs::metadata(&path).with_context(|| format!("stat {}", path.display()))?;
        stats.files += 1;
        stats.bytes = stats.bytes.saturating_add(metadata.len());
        let file = File::open(&path).with_context(|| format!("open {}", path.display()))?;
        let mut reader = BufReader::new(file);
        let mut line = Vec::new();
        let mut call_tools = BTreeMap::<String, String>::new();
        loop {
            line.clear();
            let read = reader
                .read_until(b'\n', &mut line)
                .with_context(|| format!("read {}", path.display()))?;
            if read == 0 {
                break;
            }
            stats.lines += 1;
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let kind = classify_line(&line);
            if !kind.relevant {
                continue;
            }
            stats.relevant_lines += 1;
            if kind.tool_output {
                stats.tool_output_lines += 1;
                stats.tool_output_bytes = stats.tool_output_bytes.saturating_add(line.len() as u64);
            }
            match mode {
                Mode::Count => count_relevant(&mut stats, kind),
                Mode::ValueNoOutput if kind.tool_output => count_relevant(&mut stats, kind),
                Mode::OutputMeta if kind.tool_output => parse_output_meta(&mut stats, &line),
                Mode::ValueAll | Mode::ValueNoOutput | Mode::OutputMeta => {
                    parse_value_line(&mut stats, &line, &mut call_tools, mode)?
                }
            }
        }
    }
    Ok(stats)
}

fn count_relevant(stats: &mut Stats, kind: LineKind) {
    if kind.session_meta {
        stats.session_meta += 1;
    } else if kind.tool_output {
        stats.tool_outputs += 1;
    } else if kind.tool_call {
        stats.tool_calls += 1;
    } else if kind.message {
        stats.messages += 1;
    } else if kind.reasoning {
        stats.reasoning += 1;
    } else if kind.compacted {
        stats.compacted += 1;
    } else if kind.event_msg {
        stats.event_msgs += 1;
    }
}

fn parse_value_line(
    stats: &mut Stats,
    line: &[u8],
    call_tools: &mut BTreeMap<String, String>,
    mode: Mode,
) -> Result<()> {
    let value = match serde_json::from_slice::<Value>(line) {
        Ok(value) => value,
        Err(_) => {
            stats.failed_lines += 1;
            return Ok(());
        }
    };
    stats.parsed_lines += 1;
    let entry_type = value.get("type").and_then(Value::as_str).unwrap_or("");
    match entry_type {
        "session_meta" => stats.session_meta += 1,
        "compacted" => stats.compacted += 1,
        "event_msg" => stats.event_msgs += 1,
        "response_item" => {
            let Some(payload) = value.get("payload") else {
                return Ok(());
            };
            match payload.get("type").and_then(Value::as_str).unwrap_or("") {
                "message" => parse_message(stats, payload),
                "reasoning" => stats.reasoning += 1,
                "function_call" | "custom_tool_call" | "web_search_call" | "tool_search_call" => {
                    parse_tool_call(stats, payload, call_tools);
                }
                "function_call_output" | "custom_tool_call_output" | "tool_search_output" => {
                    parse_tool_output(stats, payload, call_tools, mode);
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_message(stats: &mut Stats, payload: &Value) {
    let role = payload.get("role").and_then(Value::as_str).unwrap_or("");
    if matches!(role, "developer" | "system") {
        return;
    }
    stats.messages += 1;
    if role == "user" {
        stats.user_messages += 1;
    } else if role == "assistant" {
        stats.assistant_messages += 1;
    }
    if let Some(text) = payload.get("content").and_then(content_text_len) {
        stats.message_chars += text;
    }
}

fn parse_tool_call(stats: &mut Stats, payload: &Value, call_tools: &mut BTreeMap<String, String>) {
    stats.tool_calls += 1;
    let tool = tool_name(payload);
    if let Some(call_id) = payload.get("call_id").and_then(Value::as_str) {
        call_tools.insert(call_id.to_owned(), tool);
    }
    if let Some(argument_value) = payload
        .get("arguments")
        .or_else(|| payload.get("input"))
        .or_else(|| payload.get("action"))
        .or_else(|| payload.get("execution"))
    {
        stats.tool_call_arg_chars += value_preview_len(argument_value, 4_000);
    }
}

fn parse_tool_output(
    stats: &mut Stats,
    payload: &Value,
    call_tools: &BTreeMap<String, String>,
    mode: Mode,
) {
    stats.tool_outputs += 1;
    let call_id = payload.get("call_id").and_then(Value::as_str);
    let tool = call_id
        .and_then(|call_id| call_tools.get(call_id))
        .cloned()
        .unwrap_or_else(|| tool_name(payload));
    if matches!(tool.as_str(), "exec_command" | "shell" | "bash" | "command") {
        stats.command_outputs += 1;
    }
    if mode == Mode::OutputMeta {
        return;
    }
    if let Some(output_value) = payload
        .get("output")
        .or_else(|| payload.get("tools"))
        .or_else(|| payload.get("result"))
    {
        let preview_len = value_preview_len(output_value, 4_000);
        stats.tool_output_preview_chars += preview_len;
        if let Some(text) = output_value.as_str() {
            if text.contains("Process exited with code ") {
                stats.exit_codes += 1;
            }
            if text.contains("Wall time: ") {
                stats.wall_times += 1;
            }
        }
    }
}

fn parse_output_meta(stats: &mut Stats, line: &[u8]) {
    stats.tool_outputs += 1;
    if contains_bytes(line, b"exec_command")
        || contains_bytes(line, b"shell")
        || contains_bytes(line, b"bash")
        || contains_bytes(line, b"command")
    {
        stats.command_outputs += 1;
    }
    if contains_bytes(line, b"Process exited with code ") {
        stats.exit_codes += 1;
    }
    if contains_bytes(line, b"Wall time: ") {
        stats.wall_times += 1;
    }
    stats.tool_output_preview_chars += 4_000.min(line.len());
}

#[derive(Debug, Clone, Copy, Default)]
struct LineKind {
    relevant: bool,
    session_meta: bool,
    message: bool,
    tool_call: bool,
    tool_output: bool,
    reasoning: bool,
    compacted: bool,
    event_msg: bool,
}

fn classify_line(line: &[u8]) -> LineKind {
    let mut kind = LineKind::default();
    if contains_bytes(line, br#""type":"session_meta""#) {
        kind.relevant = true;
        kind.session_meta = true;
        return kind;
    }
    if contains_bytes(line, br#""type":"compacted""#) {
        kind.relevant = true;
        kind.compacted = true;
        return kind;
    }
    if contains_bytes(line, br#""type":"event_msg""#) {
        kind.relevant = true;
        kind.event_msg = true;
        return kind;
    }
    if !contains_bytes(line, br#""type":"response_item""#) {
        return kind;
    }
    if contains_bytes(line, br#""type":"message""#)
        && (contains_bytes(line, br#""role":"user""#)
            || contains_bytes(line, br#""role":"assistant""#))
    {
        kind.relevant = true;
        kind.message = true;
    } else if contains_bytes(line, br#""type":"function_call_output""#)
        || contains_bytes(line, br#""type":"custom_tool_call_output""#)
        || contains_bytes(line, br#""type":"tool_search_output""#)
    {
        kind.relevant = true;
        kind.tool_output = true;
    } else if contains_bytes(line, br#""type":"function_call""#)
        || contains_bytes(line, br#""type":"custom_tool_call""#)
        || contains_bytes(line, br#""type":"web_search_call""#)
        || contains_bytes(line, br#""type":"tool_search_call""#)
    {
        kind.relevant = true;
        kind.tool_call = true;
    } else if contains_bytes(line, br#""type":"reasoning""#) {
        kind.relevant = true;
        kind.reasoning = true;
    }
    kind
}

fn content_text_len(value: &Value) -> Option<usize> {
    match value {
        Value::String(text) => Some(text.chars().count()),
        Value::Array(blocks) => Some(
            blocks
                .iter()
                .filter_map(|block| {
                    block
                        .get("text")
                        .or_else(|| block.get("input_text"))
                        .or_else(|| block.get("output_text"))
                        .or_else(|| block.get("summary_text"))
                        .or_else(|| block.get("content"))
                        .and_then(Value::as_str)
                        .map(|text| text.chars().count())
                })
                .sum(),
        ),
        Value::Object(_) => serde_json::to_string(value).ok().map(|text| text.len()),
        _ => None,
    }
}

fn tool_name(payload: &Value) -> String {
    payload
        .get("name")
        .or_else(|| payload.get("tool"))
        .and_then(Value::as_str)
        .unwrap_or("tool")
        .to_owned()
}

fn value_preview_len(value: &Value, max_chars: usize) -> usize {
    match value {
        Value::String(text) => text.chars().take(max_chars).count(),
        Value::Null => 0,
        _ => serde_json::to_string(value)
            .map(|text| text.chars().take(max_chars).count())
            .unwrap_or(0),
    }
}

fn collect_jsonl_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if root.is_file() {
        if root.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
            paths.push(root.to_path_buf());
        }
        return Ok(());
    }
    for entry in fs::read_dir(root).with_context(|| format!("read dir {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_jsonl_paths(&path, paths)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl")
            && (file_type.is_file()
                || (file_type.is_symlink()
                    && fs::metadata(&path)
                        .map(|metadata| metadata.is_file())
                        .unwrap_or(false)))
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn apply_bounds(
    paths: &mut Vec<PathBuf>,
    max_files: Option<usize>,
    max_bytes: Option<u64>,
) -> Result<()> {
    if max_files.is_none() && max_bytes.is_none() {
        return Ok(());
    }
    let mut selected = Vec::new();
    let mut bytes = 0u64;
    for path in paths.iter().rev() {
        if max_files.is_some_and(|limit| selected.len() >= limit) {
            continue;
        }
        let len = fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        if max_bytes.is_some_and(|limit| bytes.saturating_add(len) > limit) {
            continue;
        }
        bytes = bytes.saturating_add(len);
        selected.push(path.clone());
    }
    selected.sort();
    *paths = selected;
    Ok(())
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
