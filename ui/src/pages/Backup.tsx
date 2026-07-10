// ============================================================================
// Backup.tsx -- Backup Jobs page (three-column layout)
//
// Data flow:
//   useEffect -> listJobs() -> job list (middle column)
//   Click job or [+ New Job] -> PlanForm in right column
//   Delete button -> confirmation -> deleteJobConfig() -> refresh
//
// States handled:
//   - Loading: skeleton cards
//   - Error:   error card with retry button
//   - Empty:   guidance screen with CTA to create first job
//   - Data:    three-column layout: job list | config panel
// ============================================================================

import React, { useEffect, useState, useCallback } from "react";
import Badge from "../components/common/Badge";
import Button from "../components/common/Button";
import EmptyState from "../components/feedback/EmptyState";
import ErrorState from "../components/feedback/ErrorState";
import { theme } from "../theme";
import {
  listJobs,
  runBackup,
  BackupJobViewFormatted,
} from "../api/backupApi";
import PathInput from "../components/common/PathInput";
import {
  listJobConfigs,
  createJobConfig,
  updateJobConfig,
  deleteJobConfig,
  JobConfigView,
  JobConfigRequest,
} from "../api/configApi";
import {
  listSchedules,
  ScheduleProfileView,
} from "../api/scheduleApi";

// ---------------------------------------------------------------------------
// Inline SVG icons
// ---------------------------------------------------------------------------

function FolderIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5c0-1.1.9-2 2-2h5l2 3h9a2 2 0 012 2v11z"/></svg>); }
function PlayIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>); }
function PlusIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>); }
function TrashIcon() { return (<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>); }
function ShieldIcon() { return (<svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>); }
function AlertIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>); }
function RefreshIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/></svg>); }
function CheckCircleIcon() { return (<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={theme.colors.success} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>); }

// ---------------------------------------------------------------------------
// Status helpers
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
// Skeleton card
// ---------------------------------------------------------------------------

function SkeletonPlanCard() {
  const sk = { height: "20px", borderRadius: theme.radius.sm, background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" };
  return (<div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, padding: theme.spacing.lg }}>
    <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "16px" }}>
      <div style={{ width: "140px", ...sk }} />
      <div style={{ width: "70px", ...sk, height: "22px", borderRadius: theme.radius.full }} />
    </div>
    <div style={{ display: "flex", gap: "24px" }}>
      <div style={{ flex: 1, ...sk }} /><div style={{ flex: 1, ...sk }} />
    </div>
  </div>);
}

// ---------------------------------------------------------------------------
// PlanForm -- Create / Edit backup job configuration (right panel)
// ---------------------------------------------------------------------------

interface PlanFormProps { initial?: JobConfigView; onSave: (request: JobConfigRequest) => Promise<void>; onCancel: () => void; saving: boolean; error?: string | null; }

function PlanForm({ initial, onSave, onCancel, saving, error }: PlanFormProps) {
  const [name, setName] = useState(initial?.name ?? "");
  const [source, setSource] = useState(initial?.source ?? "");
  const [dest, setDest] = useState(initial?.dest ?? "");
  const [compress, setCompress] = useState(initial?.compress ?? true);
  const [keepCount, setKeepCount] = useState(initial?.retention_keep_count?.toString() ?? "");
  const [keepDays, setKeepDays] = useState(initial?.retention_keep_days?.toString() ?? "");
  const [scheduleId, setScheduleId] = useState<string | null>(initial?.schedule_id ?? null);
  const [availableSchedules, setAvailableSchedules] = useState<ScheduleProfileView[]>([]);
  const [formError, setFormError] = useState<string | null>(null);

  useEffect(() => { listSchedules().then((s) => setAvailableSchedules(s)).catch(() => {}); }, []);

  const handleSubmit = async () => {
    setFormError(null);
    if (!name.trim()) { setFormError("Plan name is required"); return; }
    if (!source.trim()) { setFormError("Source path is required"); return; }
    if (!dest.trim()) { setFormError("Destination path is required"); return; }
    const request: JobConfigRequest = { name: name.trim(), source: source.trim(), dest: dest.trim(), compress, retention_keep_count: keepCount ? parseInt(keepCount) || null : null, retention_keep_days: keepDays ? parseInt(keepDays) || null : null, schedule_id: scheduleId };
    await onSave(request);
  };

  const inputStyle: React.CSSProperties = { width: "100%", padding: "10px 12px", background: theme.colors.background, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.md, color: theme.colors.textPrimary, fontSize: theme.font.sizeMd, fontFamily: theme.font.family, outline: "none", boxSizing: "border-box" };
  const labelStyle: React.CSSProperties = { display: "block", fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, marginBottom: "6px", fontWeight: 500 };

  return (
    <div>
      <div style={{ marginBottom: "20px" }}>
        <h3 style={{ fontSize: theme.font.sizeXl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>{initial ? "Edit Backup Plan" : "Create Backup Plan"}</h3>
        <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "4px", marginBottom: 0 }}>{initial ? "Update the backup plan configuration." : "Configure a new backup job to protect your data."}</p>
      </div>

      {(formError || error) && (
        <div style={{ padding: "10px 14px", background: theme.colors.errorDim, border: "1px solid rgba(239,68,68,0.2)", borderRadius: theme.radius.md, color: theme.colors.error, fontSize: theme.font.sizeSm, marginBottom: "16px", display: "flex", alignItems: "center", gap: "8px" }}>
          <AlertIcon /><span>{formError || error}</span>
        </div>
      )}

      <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
        <div><label style={labelStyle}>Plan Name</label><input style={inputStyle} value={name} onChange={(e: any) => setName(e.target.value)} placeholder="e.g., Documents Backup" disabled={saving} /></div>
        <div><label style={labelStyle}>Source Path</label><PathInput value={source} onChange={setSource} placeholder="C:\\Users\\YourName\\Documents" disabled={saving} /></div>
        <div><label style={labelStyle}>Destination Path</label><PathInput value={dest} onChange={setDest} placeholder="D:\\Backups" disabled={saving} /></div>
        <div><label style={{ ...labelStyle, marginBottom: "8px" }}>Compression</label>
          <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer", color: theme.colors.textPrimary, fontSize: theme.font.sizeMd }}>
            <input type="checkbox" checked={compress} onChange={(e: any) => setCompress(e.target.checked)} disabled={saving} style={{ accentColor: theme.colors.primary }} /> Enable compression (zstd)
          </label>
        </div>
        <div><label style={labelStyle}>Retention &mdash; Keep Count</label><input style={inputStyle} value={keepCount} onChange={(e: any) => setKeepCount(e.target.value)} placeholder="e.g., 7 (keep last 7 versions)" disabled={saving} /></div>
        <div><label style={labelStyle}>Retention &mdash; Keep Days</label><input style={inputStyle} value={keepDays} onChange={(e: any) => setKeepDays(e.target.value)} placeholder="e.g., 30 (keep 30 days)" disabled={saving} /></div>
        <div><label style={labelStyle}>Schedule</label>
          <select style={inputStyle} value={scheduleId ?? ""} onChange={(e: any) => setScheduleId(e.target.value || null)} disabled={saving}>
            <option value="">No schedule (manual only)</option>
            {availableSchedules.map((s) => (<option key={s.id} value={s.id}>{s.name}</option>))}
          </select>
        </div>
      </div>

      <div style={{ display: "flex", gap: "10px", marginTop: "24px" }}>
        <Button variant="primary" size="md" onClick={handleSubmit} disabled={saving}>{saving ? "Saving..." : initial ? "Save Changes" : "Create Plan"}</Button>
        <Button variant="secondary" size="md" onClick={onCancel} disabled={saving}>Cancel</Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// ConfirmDeleteDialog
// ---------------------------------------------------------------------------

function ConfirmDeleteDialog({ name, onConfirm, onCancel, deleting }: { name: string; onConfirm: () => void; onCancel: () => void; deleting: boolean }) {
  return (
    <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.6)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }}>
      <div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.xl, padding: "28px 32px", maxWidth: "420px", width: "100%", textAlign: "center" }}>
        <div style={{ width: "48px", height: "48px", borderRadius: "50%", background: theme.colors.errorDim, display: "flex", alignItems: "center", justifyContent: "center", margin: "0 auto 16px auto", fontSize: "24px", color: theme.colors.error }}><TrashIcon /></div>
        <h3 style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, margin: "0 0 8px 0" }}>Delete Backup Plan</h3>
        <p style={{ fontSize: theme.font.sizeMd, color: theme.colors.textSecondary, margin: "0 0 4px 0", lineHeight: 1.5 }}>Are you sure you want to delete the backup plan &ldquo;<strong>{name}</strong>&rdquo;?</p>
        <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, margin: "0 0 20px 0", lineHeight: 1.5 }}>Only the plan configuration will be removed. Backup data on disk will be preserved.</p>
        <div style={{ display: "flex", gap: "10px", justifyContent: "center" }}>
          <Button variant="secondary" size="md" onClick={onCancel} disabled={deleting}>Cancel</Button>
          <Button variant="primary" size="md" onClick={onConfirm} disabled={deleting}>{deleting ? "Deleting..." : "Delete Plan"}</Button>
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Toast notification
// ---------------------------------------------------------------------------

function Toast({ message, onClose }: { message: string; onClose: () => void }) {
  useEffect(() => { const t = setTimeout(onClose, 3000); return () => clearTimeout(t); }, [onClose]);
  return (
    <div style={{ position: "fixed", bottom: "24px", right: "24px", background: theme.colors.successDim, border: "1px solid " + theme.colors.success + "40", borderRadius: theme.radius.lg, padding: "12px 20px", display: "flex", alignItems: "center", gap: "10px", color: theme.colors.success, fontSize: theme.font.sizeMd, fontWeight: 500, zIndex: 1100, boxShadow: "0 4px 24px rgba(0,0,0,0.3)" }}>
      <CheckCircleIcon /><span>{message}</span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Job Card component
// ---------------------------------------------------------------------------

function JobCard({ job, selected, onRun, onSelect, onDelete, running }: { job: BackupJobViewFormatted; selected: boolean; onRun: () => void; onSelect: () => void; onDelete: () => void; running: boolean }) {
  const sb = statusBadge(job.statusLabel);
  return (
    <div style={{ background: selected ? "rgba(59,130,246,0.08)" : theme.colors.panel, border: "1px solid " + (selected ? theme.colors.primary : theme.colors.panelBorder), borderRadius: theme.radius.lg, padding: theme.spacing.lg, cursor: "pointer", transition: theme.transition.fast }}
      onClick={onSelect}
      onMouseEnter={(e: any) => { if (!selected) (e.currentTarget as HTMLElement).style.borderColor = theme.colors.primary; }}
      onMouseLeave={(e: any) => { if (!selected) (e.currentTarget as HTMLElement).style.borderColor = theme.colors.panelBorder; }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <FolderIcon />
          <h3 style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>{job.name}</h3>
          <span><Badge variant={sb.variant}>{sb.label}</Badge></span>
        </div>
        <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
          <button onClick={(e: any) => { e.stopPropagation(); onDelete(); }}
            title="Delete this backup plan (backup data preserved)"
            style={{ background: "rgba(239,68,68,0.1)", border: "1px solid rgba(239,68,68,0.2)", borderRadius: theme.radius.md, color: theme.colors.error, padding: "6px 10px", cursor: "pointer", display: "flex", alignItems: "center", gap: "4px", fontSize: theme.font.sizeSm, fontFamily: theme.font.family, transition: theme.transition.fast }}
            onMouseEnter={(e: any) => { (e.currentTarget as HTMLElement).style.background = theme.colors.errorDim; }}
            onMouseLeave={(e: any) => { (e.currentTarget as HTMLElement).style.background = "rgba(239,68,68,0.1)"; }}>
            <TrashIcon /> Delete
          </button>
          <Button variant="primary" size="sm" onClick={onRun} disabled={!job.canRun || running} icon={<PlayIcon />}>{running ? "Running..." : "Run"}</Button>
        </div>
      </div>
      <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, marginBottom: "10px" }}>
        <span style={{ color: theme.colors.textMuted }}>Source:</span> {job.source}
        <span style={{ margin: "0 8px", color: theme.colors.textMuted }}>&rarr;</span>
        <span style={{ color: theme.colors.textMuted }}>Dest:</span> {job.dest}
      </div>
      <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
        {job.lastBackupTime ? (<>{lastBackupStatusText(job.lastBackupStatus)} &middot; {job.lastBackupFiles} files &middot; {job.lastBackupBytes}<span style={{ marginLeft: "8px", opacity: 0.6 }}>at {job.lastBackupTime}</span></>) : "No backup history"}
        {job.compress && <Badge variant="info">Compressed</Badge>}
      </div>
      <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginTop: "6px" }}>Retention: <span style={{ color: theme.colors.textSecondary }}>{job.retention}</span></div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Empty right panel placeholder
// ---------------------------------------------------------------------------

function EmptyRightPanel({ onNew }: { onNew: () => void }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: "100%", textAlign: "center", padding: "40px 20px" }}>
      <div style={{ opacity: 0.3, marginBottom: "16px" }}><ShieldIcon /></div>
      <p style={{ fontSize: theme.font.sizeMd, color: theme.colors.textMuted, margin: "0 0 16px 0" }}>Select a backup plan from the list to edit, or create a new one.</p>
      <Button variant="primary" size="md" onClick={onNew} icon={<PlusIcon />}>New Backup Plan</Button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main Backup Page
// ---------------------------------------------------------------------------

export default function Backup() {
  const [jobs, setJobs] = useState<BackupJobViewFormatted[]>([]);
  const [plans, setPlans] = useState<JobConfigView[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [runningJob, setRunningJob] = useState<string | null>(null);
  const [editingPlanName, setEditingPlanName] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [showConfirmDelete, setShowConfirmDelete] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const fetchJobs = useCallback(() => {
    setLoading(true);
    setError(null);
    listJobs()
      .then((j) => { setJobs(j); return listJobConfigs(); })
      .then((c) => { setPlans(c); setLoading(false); })
      .catch(() => { setError("Failed to load backup jobs"); setLoading(false); });
  }, []);

  const editingPlan = editingPlanName ? plans.find((p) => p.name === editingPlanName) ?? null : null;

  useEffect(() => { fetchJobs(); }, [fetchJobs]);

  const openCreateForm = useCallback(() => { setEditingPlanName(null); setFormError(null); setShowForm(true); }, []);
  const openEditForm = useCallback((name: string) => { setEditingPlanName(name); setFormError(null); setShowForm(true); }, []);
  const closeForm = useCallback(() => { setShowForm(false); setEditingPlanName(null); setFormError(null); }, []);

  const handleSave = useCallback(async (request: JobConfigRequest) => {
    setSaving(true); setFormError(null);
    try {
      if (editingPlanName) { await updateJobConfig(editingPlanName, request); setToastMessage("Plan updated: " + request.name); }
      else { await createJobConfig(request); setToastMessage("Plan created: " + request.name); }
      closeForm();
      await fetchJobs();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Operation failed";
      setFormError(msg);
    } finally { setSaving(false); }
  }, [editingPlanName, closeForm, fetchJobs]);

  const handleDelete = useCallback(async () => {
    if (!showConfirmDelete) return;
    setDeleting(true);
    try {
      await deleteJobConfig(showConfirmDelete);
      setToastMessage("Deleted: " + showConfirmDelete + " (backup data preserved)");
      if (editingPlanName === showConfirmDelete) closeForm();
      setShowConfirmDelete(null);
      await fetchJobs();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Delete failed";
      setFormError(msg);
      setShowConfirmDelete(null);
    } finally { setDeleting(false); }
  }, [showConfirmDelete, editingPlanName, closeForm, fetchJobs]);

  const handleRunBackup = useCallback(async (jobName: string) => {
    setRunningJob(jobName);
    try {
      await runBackup(jobName);
      await fetchJobs();
    } catch {
      // Error logged by API
    } finally { setRunningJob(null); }
  }, [fetchJobs]);

  // ---- Loading ----
  if (loading) {
    return (<div style={{ padding: theme.spacing.lg }}>
      <div style={{ marginBottom: "24px" }}><div style={{ width: "200px", height: "28px", borderRadius: theme.radius.sm, ...{ background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" } }} /></div>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}><SkeletonPlanCard /><SkeletonPlanCard /><SkeletonPlanCard /></div>
    </div>);
  }

  // ---- Error ----
  if (error) {
    return (<div style={{ padding: theme.spacing.lg }}><ErrorState title="Failed to load backup jobs" message={error} onRetry={fetchJobs} /></div>);
  }

  // ---- Data: Three-column layout ----
  const activeCount = jobs.filter((j) => j.status === "Active").length;

  return (
    <div style={{ display: "flex", height: "100%", gap: "16px", padding: theme.spacing.lg }}>

      {/* Middle: Job list */}
      <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px", flexShrink: 0 }}>
          <div>
            <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>Backup Jobs</h2>
            <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>{jobs.length > 0 ? jobs.length + " job" + (jobs.length > 1 ? "s" : "") + " configured \u00B7 " + activeCount + " active" : "No backup plans configured"}</p>
          </div>
          <div style={{ display: "flex", gap: "8px" }}>
            <Button variant="secondary" size="sm" onClick={fetchJobs} icon={<RefreshIcon />}>Refresh</Button>
            <Button variant="primary" size="sm" onClick={openCreateForm} icon={<PlusIcon />}>New Plan</Button>
          </div>
        </div>

        <div style={{ flex: 1, overflowY: "auto", display: "flex", flexDirection: "column", gap: "10px" }}>
          {jobs.length === 0 ? (
            <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
              <EmptyState title="No Backup Plans Configured" description="Create a backup plan to protect your important data." primaryAction={{ label: "Create Plan", onClick: openCreateForm }} />
            </div>
          ) : jobs.map((job) => (
            <JobCard key={job.name} job={job} selected={editingPlanName === job.name}
              onRun={() => handleRunBackup(job.name)} onSelect={() => { openEditForm(job.name); }}
              onDelete={() => setShowConfirmDelete(job.name)} running={runningJob === job.name} />
          ))}
        </div>
      </div>

      {/* Right: Config panel */}
      <div style={{ flex: 1, minWidth: 0, background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.xl, padding: theme.spacing.xl, overflowY: "auto" }}>
        {showForm ? (
          <PlanForm initial={editingPlan ?? undefined} onSave={handleSave} onCancel={closeForm} saving={saving} error={formError} />
        ) : (
          <EmptyRightPanel onNew={openCreateForm} />
        )}
      </div>

      {/* Delete confirmation */}
      {showConfirmDelete && (
        <ConfirmDeleteDialog name={showConfirmDelete} onConfirm={handleDelete} onCancel={() => setShowConfirmDelete(null)} deleting={deleting} />
      )}

      {/* Toast */}
      {toastMessage && (<Toast message={toastMessage} onClose={() => setToastMessage(null)} />)}
    </div>
  );
}
