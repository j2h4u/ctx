export type TagCount = {
  tag: string;
  count: number;
};

export type DashboardSummary = {
  record_count: number;
  evidence_count: number;
  linked_pr_count: number;
  tags: TagCount[];
};

export type PrivacySummary = {
  default_redacted: boolean;
  raw_transcripts_withheld: number;
  redacted_previews: number;
  withheld_links: number;
  local_paths_redacted: boolean;
};

export type DashboardRecord = {
  id: string;
  title: string;
  body: string;
  tags: string[];
  kind: string;
  workspace?: string | null;
  pr_url?: string | null;
  created_at: string;
  updated_at: string;
};

export type EvidenceCommand = {
  id: string;
  record_id?: string | null;
  command: string;
  exit_code: number;
  duration_ms: number;
  started_at: string;
  output_preview?: string | null;
};

export type PullRequest = {
  url: string;
  title?: string | null;
  state?: string | null;
  head_ref?: string | null;
  base_ref?: string | null;
};

export type DashboardData = {
  schema_version: number;
  product: string;
  share_safe: boolean;
  summary: DashboardSummary;
  privacy: PrivacySummary;
  views: string[];
  records: DashboardRecord[];
  commands: EvidenceCommand[];
  sessions: Record<string, unknown>[];
  runs: Record<string, unknown>[];
  events: Record<string, unknown>[];
  vcs_workspaces: Record<string, unknown>[];
  vcs_changes: Record<string, unknown>[];
  pull_requests: PullRequest[];
  artifacts: Record<string, unknown>[];
  evidence_metadata: Record<string, unknown>[];
  files_touched: Record<string, unknown>[];
  summaries: Record<string, unknown>[];
  status: {
    export_mode: string;
    local_only: boolean;
    javascript_app: string;
    data_contract: string;
    search_command: string;
  };
};
