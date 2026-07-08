// ============================================================================
// Backup.tsx -- Backup Jobs page
//
// Data flow:
//   useEffect -> listJobs() -> BackupJobViewFormatted[] -> render
//
// States handled:
//   - Loading: skeleton cards that mirror the job card layout
//   - Error:   error card with reasons + retry button
//   - Empty:   guidance screen when no backup jobs are configured
//   - Data:    job list with status, details, and run action
//
// View modes:
//   - List: shows all jobs as cards (default)
//   - Detail: shows expanded view of a single job (selected)
// ============================================================================

import { useEffect, useState, useCallback } from "react";
import Badge from "../components/common/Badge";
import Button from "../components/common/Button";
import EmptyState from "../components/feedback/EmptyState";
import ErrorState from "../components/feedback/ErrorState";
import { theme } from "../theme";
import {
  listJobs,
  runBackup,
  BackupJobViewFormatted,
  BackupResult,
} from "../api/backupApi";

// ---------------------------------------------------------------------------
// Inline SVG icons
// ---------------------------------------------------------------------------

function FolderIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5c0-1.1.9-2 2-2h5l2 3h9a2 2 0 012 2v11z"/>
    </svg>
  );
}

function PlayIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="5 3 19 12 5 21 5 3"/>
    </svg>
  );
}

function RefreshIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="23 4 23 10 17 10"/>
      <path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/>
    </svg>
  );
}

function ArrowLeftIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="19" y1="12" x2="5" y2="12"/>
      <polyline points="12 19 5 12 12 5"/>
    </svg>
  );
}

function CheckCircleIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={theme.colors.success} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/>
      <polyline points="22 4 12 14.01 9 11.01"/>
    </svg>
  );
}

function XCircleIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={theme.colors.error} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10"/>
      <line x1="15" y1="9" x2="9" y2="15"/>
      <line x1="9" y1="9" x2="15" y2="15"/>
    </svg>
  );
}

// ---------------------------------------------------------------------------
// Status badge mapping
// ---------------------------------------------------------------------------

type BadgeVariant = "success" | "warning" | "error" | "info" | "neutral";

function statusBadge(status: string): { variant: BadgeVariant; label: string } {
  switch (status) {
    case "Active":        return { variant: "success", label: "Active" };
    case "NeverRun":      return { variant: "neutral", label: "Never Run" };
    case "Error":         return { variant: "error",   label: "Error" };
    case "Misconfigured": return { variant: "warning", label: "Misconfigured" };
    default:              return { variant: "info",    label: status };
  }
}

function lastBackupStatusText(status: string | null): string {
  if (status === "success") return "Last backup succeeded";
  if (status === "failure") return "Last backup failed";
  return "No backup history";
}

// ---------------------------------------------------------------------------
// Skeleton card for loading state
// ---------------------------------------------------------------------------

function SkeletonJobCard() {
  const sk = {
    height: "20px",
    borderRadius: theme.radius.sm,
    background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)",
    backgroundSize: "200% 100%",
    animation: "skeletonPulse 1.5s infinite",
  };
  return (
    <div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, padding: theme.spacing.lg }}>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "16px" }}>
        <div style={{ width: "140px", ...sk }} />
        <div style={{ width: "70px", ...sk, height: "22px", borderRadius: theme.radius.full }} />
      </div>
      <div style={{ display: "flex", gap: "24px" }}>
        <div style={{ flex: 1, ...sk }} />
        <div style={{ flex: 1, ...sk }} />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Job Card component
// ---------------------------------------------------------------------------

function JobCard({
  job,
  onRun,
  onSelect,
  running,
}: {
  job: BackupJobViewFormatted;
  onRun: () => void;
  onSelect: () => void;
  running: boolean;
}) {
  const sb = statusBadge(job.statusLabel);

  return (
    <div className="dashboard-card" onClick={onSelect} style={{
      background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg, padding: theme.spacing.lg, cursor: "pointer",
    }}>
      {/* Header: name + badge */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "14px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <div style={{ width: "36px", height: "36px", borderRadius: theme.radius.md, background: theme.colors.primaryDim, display: "flex", alignItems: "center", justifyContent: "center", color: theme.colors.primary, flexShrink: 0 }}>
            <FolderIcon />
          </div>
          <div>
            <div style={{ fontSize: theme.font.sizeMd, fontWeight: 600, color: theme.colors.textPrimary }}>{job.name}</div>
            <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>{lastBackupStatusText(job.lastBackupStatus)}</div>
          </div>
        </div>
        <Badge variant={sb.variant}>{sb.label}</Badge>
      </div>

      {/* Source + Dest */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
        <div>
          <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "2px" }}>Source</div>
          <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, fontFamily: theme.font.mono, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{job.source}</div>
        </div>
        <div>
          <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "2px" }}>Destination</div>
          <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, fontFamily: theme.font.mono, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{job.dest}</div>
        </div>
      </div>

      {/* Footer: retention + action */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: "14px", paddingTop: "12px", borderTop: "1px solid " + theme.colors.divider }}>
        <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
          Retention: <span style={{ color: theme.colors.textSecondary }}>{job.retention}</span>
        </div>
        <Button variant="primary" size="sm" onClick={onRun} disabled={!job.canRun || running} icon={<PlayIcon />}>
          {running ? "Running..." : "Run"}
        </Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Detail View for a single job
// ---------------------------------------------------------------------------

function JobDetailView({ job, onBack, onRun, result, running }: {
  job: BackupJobViewFormatted;
  onBack: () => void;
  onRun: () => void;
  result: BackupResult | null;
  running: boolean;
}) {
  const sb = statusBadge(job.statusLabel);

  return (
    <div>
      <button onClick={onBack} style={{
        display: "inline-flex", alignItems: "center", gap: "6px",
        background: "transparent", border: "none", color: theme.colors.textSecondary,
        fontSize: theme.font.sizeSm, cursor: "pointer", padding: "4px 0", marginBottom: "16px",
      }}>
        <ArrowLeftIcon /> Back to all jobs
      </button>

      <div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, padding: theme.spacing.lg }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "20px" }}>
          <div>
            <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "4px" }}>
              <span style={{ fontSize: theme.font.sizeXl, fontWeight: 700, color: theme.colors.textPrimary }}>{job.name}</span>
              <Badge variant={sb.variant}>{sb.label}</Badge>
            </div>
            <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted }}>{lastBackupStatusText(job.lastBackupStatus)}</div>
          </div>
          <Button variant="primary" onClick={onRun} disabled={!job.canRun || running} icon={<PlayIcon />}>
            {running ? "Running..." : "Run Backup Now"}
          </Button>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "16px" }}>
          <div>
            <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "4px" }}>Source Path</div>
            <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, fontFamily: theme.font.mono }}>{job.source}</div>
          </div>
          <div>
            <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "4px" }}>Destination Path</div>
            <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, fontFamily: theme.font.mono }}>{job.dest}</div>
          </div>
          <div>
            <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "4px" }}>Retention</div>
            <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>{job.retention}</div>
          </div>
          <div>
            <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "4px" }}>Compression</div>
            <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>{job.compress ? "Enabled (zstd)" : "Disabled"}</div>
          </div>
          <div>
            <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "4px" }}>Last Backup</div>
            <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
              {job.lastBackupTime ? new Date(job.lastBackupTime).toLocaleString() : "Never"}
            </div>
          </div>
          <div>
            <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "4px" }}>Last Backup Size</div>
            <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>{job.lastBackupBytes}</div>
          </div>
        </div>
      </div>

      {/* Run result */}
      {result && (
        <div style={{ marginTop: "16px", background: result.status === "success" ? theme.colors.successDim : theme.colors.errorDim, border: "1px solid " + (result.status === "success" ? "rgba(34,197,94,0.2)" : "rgba(239,68,68,0.2)"), borderRadius: theme.radius.md, padding: "12px 16px", display: "flex", alignItems: "center", gap: "10px" }}>
          {result.status === "success" ? <CheckCircleIcon /> : <XCircleIcon />}
          <div>
            <div style={{ fontSize: theme.font.sizeMd, fontWeight: 600, color: result.status === "success" ? theme.colors.success : theme.colors.error }}>
              {result.status === "success" ? "Backup completed" : "Backup failed"}
            </div>
            <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
              {result.backup_id} &middot; {result.duration_ms}ms
              {result.error && <span> &middot; {result.error}</span>}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main Backup Page
// ---------------------------------------------------------------------------

export default function Backup() {
  const [jobs, setJobs] = useState<BackupJobViewFormatted[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedJob, setSelectedJob] = useState<string | null>(null);
  const [runningJob, setRunningJob] = useState<string | null>(null);
  const [runResult, setRunResult] = useState<BackupResult | null>(null);

  const fetchJobs = useCallback(() => {
    setLoading(true);
    setError(null);

    listJobs()
      .then((jobs) => {
        setJobs(jobs);
        setLoading(false);
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : "Failed to load backup jobs";
        setError(msg);
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    fetchJobs();
  }, [fetchJobs]);

  const handleRunBackup = useCallback(async (jobName: string) => {
    setRunningJob(jobName);
    setRunResult(null);

    try {
      const res = await runBackup(jobName);
      setRunResult(res);
      await fetchJobs();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Backup failed";
      setRunResult({
        backup_id: "error",
        timestamp: new Date().toISOString(),
        file_count: 0,
        total_bytes: 0,
        duration_ms: 0,
        status: "failure",
        error: msg,
      });
    } finally {
      setRunningJob(null);
    }
  }, [fetchJobs]);

  const selectedJobData = selectedJob ? jobs.find((j) => j.name === selectedJob) ?? null : null;

  // ---- Loading ----
  if (loading) {
    return (
      <div style={{ padding: theme.spacing.lg, maxWidth: "900px" }}>
        <div style={{ marginBottom: "24px" }}>
          <div style={{ width: "200px", height: "28px", borderRadius: theme.radius.sm, ...{ background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" } }} />
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <SkeletonJobCard /><SkeletonJobCard /><SkeletonJobCard />
        </div>
      </div>
    );
  }

  // ---- Error ----
  if (error) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <ErrorState title="Failed to load backup jobs" message={error} onRetry={fetchJobs} />
      </div>
    );
  }

  // ---- Empty ----
  if (jobs.length === 0) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <EmptyState
          title="No Backup Jobs Configured"
          description="Create a backup job to protect your important data. Configure your first backup plan in Settings."
          primaryAction={{ label: "Create Backup Job" }}
        />
      </div>
    );
  }

  // ---- Detail View ----
  if (selectedJobData) {
    return (
      <div style={{ padding: theme.spacing.lg, maxWidth: "900px" }}>
        <JobDetailView
          job={selectedJobData}
          onBack={() => { setSelectedJob(null); setRunResult(null); }}
          onRun={() => handleRunBackup(selectedJobData.name)}
          result={runResult}
          running={runningJob === selectedJobData.name}
        />
      </div>
    );
  }

  // ---- List View ----
  const activeCount = jobs.filter((j) => j.status === "Active").length;

  return (
    <div style={{ padding: theme.spacing.lg, maxWidth: "900px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "24px" }}>
        <div>
          <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>Backup Jobs</h2>
          <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>
            {jobs.length} job{jobs.length > 1 ? "s" : ""} configured &middot; {activeCount} active
          </p>
        </div>
        <Button variant="secondary" size="sm" onClick={fetchJobs} icon={<RefreshIcon />}>Refresh</Button>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {jobs.map((job) => (
          <JobCard
            key={job.name}
            job={job}
            onRun={() => handleRunBackup(job.name)}
            onSelect={() => { setSelectedJob(job.name); setRunResult(null); }}
            running={runningJob === job.name}
          />
        ))}
      </div>
    </div>
  );
}

