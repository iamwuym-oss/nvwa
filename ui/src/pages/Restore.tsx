// ============================================================================
// Restore.tsx -- Restore management page
//
// Data flow:
//   useEffect -> listRestorePoints() -> RestorePointViewFormatted[] -> render
//   On point select -> getRestorePreview(backupId) -> RestorePreview -> render
//   On restore -> executeRestore(request) -> RestoreOperationResult -> render
//   On delete -> confirm -> deleteBackupSet(backupId) -> refresh
//
// States handled:
//   - Loading: skeleton cards for restore point list
//   - Error:   error card with reasons + retry button
//   - Empty:   guidance screen when no restore points are available
//   - List:    restore point list with selection + delete per row
//   - Preview: file list for selected backup point + restore form
//   - Running: restore in progress with spinner
//   - Result:  restore completed with success/failure summary
// ============================================================================

import { useEffect, useState, useCallback, useMemo } from "react";
import Badge from "../components/common/Badge";
import Button from "../components/common/Button";
import PathInput from "../components/common/PathInput";
import BackupTreeView from "../components/common/BackupTreeView";
import EmptyState from "../components/feedback/EmptyState";
import ErrorState from "../components/feedback/ErrorState";
import { theme } from "../theme";
import {
  listRestorePoints,
  getRestorePreview,
  executeRestore,
  deleteBackupSet,
  RestorePointViewFormatted,
  RestorePreview,
  RestoreOperationResult,
} from "../api/restoreApi";

// ---------------------------------------------------------------------------
// Inline SVG icons
// ---------------------------------------------------------------------------

function RestoreIcon() {
  return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 102.13-9.36L1 10"/></svg>);
}

function RefreshIcon() {
  return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/></svg>);
}

function ArrowLeftIcon() {
  return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>);
}

function CheckCircleIcon() {
  return (<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={theme.colors.success} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>);
}

function XCircleIcon() {
  return (<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={theme.colors.error} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>);
}

function ShieldIcon() {
  return (<svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>);
}

function TrashIcon() {
  return (<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>);
}

// ---------------------------------------------------------------------------
// ConfirmDeleteDialog
// ---------------------------------------------------------------------------

function ConfirmDeleteDialog({ timestamp, onConfirm, onCancel, deleting }: { timestamp: string; onConfirm: () => void; onCancel: () => void; deleting: boolean }) {
  return (
    <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.6)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }}>
      <div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.xl, padding: "28px 32px", maxWidth: "420px", width: "100%", textAlign: "center" }}>
        <div style={{ width: "48px", height: "48px", borderRadius: "50%", background: theme.colors.errorDim, display: "flex", alignItems: "center", justifyContent: "center", margin: "0 auto 16px auto", fontSize: "24px", color: theme.colors.error }}>
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>
        </div>
        <h3 style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, margin: "0 0 8px 0" }}>Permanently Delete Backup Set</h3>
        <p style={{ fontSize: theme.font.sizeMd, color: theme.colors.textSecondary, margin: "0 0 4px 0", lineHeight: 1.5 }}>
          Are you sure you want to delete the backup set from <strong>{timestamp}</strong>?
        </p>
        <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.error, margin: "0 0 4px 0", lineHeight: 1.5, fontWeight: 600 }}>
          This will permanently delete all backup files in this set. This action cannot be undone.
        </p>
        <p style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, margin: "0 0 20px 0", lineHeight: 1.5 }}>
          The job configuration and other backup sets will not be affected.
        </p>
        <div style={{ display: "flex", gap: "10px", justifyContent: "center" }}>
          <Button variant="secondary" size="md" onClick={onCancel} disabled={deleting}>Cancel</Button>
          <Button variant="primary" size="md" onClick={onConfirm} disabled={deleting}>{deleting ? "Deleting..." : "Delete Permanently"}</Button>
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
// Skeleton card for loading state
// ---------------------------------------------------------------------------

function SkeletonRestoreCard() {
  const sk = { height: "20px", borderRadius: theme.radius.sm, background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" };
  return (<div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, padding: theme.spacing.lg }}>
    <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "16px" }}>
      <div style={{ width: "160px", ...sk }} /><div style={{ width: "70px", ...sk, height: "22px", borderRadius: theme.radius.full }} />
    </div>
    <div style={{ display: "flex", gap: "24px", marginBottom: "12px" }}>
      <div style={{ width: "180px", ...sk }} /><div style={{ width: "120px", ...sk }} />
    </div>
    <div style={{ width: "280px", ...sk }} />
  </div>);
}

// ---------------------------------------------------------------------------
// Restore Form
// ---------------------------------------------------------------------------

interface RestoreFormProps {
  sourceRoot: string;
  onRestore: (dest: string, overwrite: boolean) => void;
  running: boolean;
}

function RestoreForm({ sourceRoot, onRestore, running }: RestoreFormProps) {
  const [dest, setDest] = useState("");
  const [overwrite, setOverwrite] = useState(false);

  return (
    <div style={{ marginTop: "12px", paddingTop: "16px", borderTop: "1px solid " + theme.colors.divider }}>
      <h4 style={{ margin: "0 0 12px", fontSize: theme.font.sizeSm, fontWeight: 600, color: theme.colors.textSecondary }}>
        Restore to
      </h4>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        <PathInput value={dest} onChange={setDest} placeholder={sourceRoot + " (default)"} disabled={running} />
        <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer", color: theme.colors.textPrimary, fontSize: theme.font.sizeSm }}>
          <input type="checkbox" checked={overwrite} onChange={(e: any) => setOverwrite(e.target.checked)} disabled={running} style={{ accentColor: theme.colors.primary }} />
          Overwrite existing files
        </label>
        <Button variant="primary" onClick={() => onRestore(dest, overwrite)} disabled={running} icon={<RestoreIcon />}>
          {running ? "Restoring..." : "Restore"}
        </Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Restore Result View
// ---------------------------------------------------------------------------

function RestoreResultView({ result, onDone }: { result: RestoreOperationResult; onDone: () => void }) {
  const isSuccess = result.status === "success";
  return (
    <div style={{ textAlign: "center", padding: "40px 20px" }}>
      <div style={{ marginBottom: "16px" }}>
        {isSuccess ? <CheckCircleIcon /> : <XCircleIcon />}
      </div>
      <h3 style={{ fontSize: theme.font.sizeXl, fontWeight: 700, color: isSuccess ? theme.colors.success : theme.colors.error, margin: "0 0 8px 0" }}>
        {isSuccess ? "Restore Complete" : "Restore Failed"}
      </h3>
      <p style={{ fontSize: theme.font.sizeMd, color: theme.colors.textSecondary, margin: "0 0 4px 0" }}>
        {result.restored_count} files restored
      </p>
      {result.skipped_count > 0 && (<p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, margin: "0 0 4px 0" }}>{result.skipped_count} files skipped</p>)}
      {result.checksum_failures > 0 && (<p style={{ fontSize: theme.font.sizeSm, color: theme.colors.error, margin: "0 0 4px 0" }}>{result.checksum_failures} checksum failures</p>)}
      {result.error && (<p style={{ fontSize: theme.font.sizeSm, color: theme.colors.error, margin: "0 0 12px 0" }}>{result.error}</p>)}
      <p style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginBottom: "16px" }}>
        Restore ID: {result.restore_id} &middot; {result.duration_ms}ms
      </p>
      <Button variant="secondary" onClick={onDone}>Back to Restore Points</Button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Format bytes helper
// ---------------------------------------------------------------------------

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + " " + units[i];
}

// ---------------------------------------------------------------------------
// Main Restore Page
// ---------------------------------------------------------------------------

export default function Restore() {
  const [points, setPoints] = useState<RestorePointViewFormatted[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedPoint, setSelectedPoint] = useState<RestorePointViewFormatted | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [preview, setPreview] = useState<RestorePreview | null>(null);
  const [selectedTreePath, setSelectedTreePath] = useState<string | null>(null);
  const [restoreRunning, setRestoreRunning] = useState(false);
  const [restoreResult, setRestoreResult] = useState<RestoreOperationResult | null>(null);
  const [expandedPlans, setExpandedPlans] = useState<Set<string>>(new Set());
  const [showConfirmDelete, setShowConfirmDelete] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const fetchPoints = useCallback(() => {
    setLoading(true);
    setError(null);
    listRestorePoints()
      .then((pts) => { setPoints(pts); setLoading(false); })
      .catch(() => { setError("Failed to load restore points"); setLoading(false); });
  }, []);

  useEffect(() => { fetchPoints(); }, [fetchPoints]);

  const togglePlan = useCallback((planName: string) => {
    setExpandedPlans((prev) => {
      const next = new Set(prev);
      if (next.has(planName)) next.delete(planName);
      else next.add(planName);
      return next;
    });
  }, []);

  const handleSelectPoint = useCallback((pt: RestorePointViewFormatted) => {
    setSelectedPoint(pt);
    setPreview(null);
    setPreviewError(null);
    setRestoreResult(null);
    setSelectedTreePath(null);

    if (pt.backupId) {
      setPreviewLoading(true);
      getRestorePreview(pt.backupId)
        .then((p) => { setPreview(p); setPreviewLoading(false); })
        .catch((err: any) => {
          const msg = err instanceof Error ? err.message : "Failed to load preview";
          setPreviewError(msg);
          setPreviewLoading(false);
        });
    }
  }, []);

  const handleBackToList = useCallback(() => {
    setSelectedPoint(null);
    setPreview(null);
    setPreviewError(null);
    setRestoreResult(null);
  }, []);

  const handleRestore = useCallback(async (dest: string, overwrite: boolean) => {
    if (!selectedPoint?.backupId) return;
    setRestoreRunning(true);
    setRestoreResult(null);
    try {
      const result = await executeRestore({
        backup_id: selectedPoint.backupId,
        dest: dest || selectedPoint.sourceRoot,
        overwrite,
      });
      setRestoreResult(result);
    } catch (err: any) {
      const msg = err instanceof Error ? err.message : "Restore failed";
      setRestoreResult({
        restore_id: "error",
        restored_count: 0,
        skipped_count: 0,
        checksum_failures: 0,
        timestamp: new Date().toISOString(),
        duration_ms: 0,
        status: "failure",
        error: msg,
      });
    } finally { setRestoreRunning(false); }
  }, [selectedPoint]);

  // ---- Delete handler ----
  const handleDelete = useCallback(async () => {
    if (!showConfirmDelete) return;
    setDeleting(true);
    try {
      await deleteBackupSet(showConfirmDelete);
      setToastMessage("Backup set deleted successfully");
      if (selectedPoint?.backupId === showConfirmDelete) handleBackToList();
      setShowConfirmDelete(null);
      await fetchPoints();
    } catch (err: any) {
      const msg = err instanceof Error ? err.message : "Delete failed";
      setToastMessage("Delete failed: " + msg);
      setShowConfirmDelete(null);
    } finally { setDeleting(false); }
  }, [showConfirmDelete, selectedPoint, handleBackToList, fetchPoints]);

  // Group points by job name
  const groupedPoints = useMemo(() => {
    const grouped = new Map<string, RestorePointViewFormatted[]>();
    for (const pt of points) {
      const key = pt.jobName || "Unknown";
      if (!grouped.has(key)) grouped.set(key, []);
      grouped.get(key)!.push(pt);
    }
    return grouped;
  }, [points]);

  const totalPointCount = points.length;
  const totalPlanCount = groupedPoints.size;

  // --- Loading ---
  if (loading) {
    return (<div style={{ padding: theme.spacing.lg }}>
      <div style={{ marginBottom: "24px" }}><div style={{ width: "200px", height: "28px", borderRadius: theme.radius.sm, background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" }} /></div>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}><SkeletonRestoreCard /><SkeletonRestoreCard /><SkeletonRestoreCard /></div>
    </div>);
  }

  // --- Error ---
  if (error) {
    return (<div style={{ padding: theme.spacing.lg }}>
      <ErrorState title="Failed to load restore points" message={error}
        reasons={["Backup configuration may be missing", "Storage location may be unavailable", "No backup history found"]}
        onRetry={fetchPoints} />
    </div>);
  }

  // --- Empty ---
  if (points.length === 0) {
    return (<div style={{ padding: theme.spacing.lg }}>
      <EmptyState icon={<ShieldIcon />} title="No Restore Points Available" description="Run a backup first to create restore points. Once backups exist, you can restore your files from this page." />
    </div>);
  }

  // --- Two-column layout ---
  return (
    <div style={{ display: "flex", gap: "16px", padding: theme.spacing.lg, height: "calc(100vh - 100px)" }}>

      {/* Left Column: Restore Points by Plan */}
      <div style={{ flex: 1, minWidth: 0, background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        {/* Header */}
        <div style={{ padding: "16px", borderBottom: "1px solid " + theme.colors.panelBorder }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <h3 style={{ margin: 0, fontSize: theme.font.sizeLg, fontWeight: 600, color: theme.colors.textPrimary }}>Restore Points</h3>
            <Button variant="secondary" onClick={fetchPoints} icon={<RefreshIcon />}>Refresh</Button>
          </div>
          <p style={{ margin: "4px 0 0", fontSize: theme.font.sizeSm, color: theme.colors.textMuted }}>{totalPointCount} point{totalPointCount !== 1 ? "s" : ""} in {totalPlanCount} plan{totalPlanCount !== 1 ? "s" : ""}</p>
        </div>

        {/* Scrollable plan list */}
        <div style={{ flex: 1, overflowY: "auto", padding: "8px" }}>
          {Array.from(groupedPoints.entries()).map(([planName, pts]) => {
            const isExpanded = expandedPlans.has(planName);
            return (
              <div key={planName} style={{ marginBottom: "4px" }}>
                {/* Plan header */}
                <div style={{ display: "flex", alignItems: "center", gap: "8px", padding: "8px 10px", borderRadius: theme.radius.sm, cursor: "pointer", background: isExpanded ? theme.colors.primaryDim : "transparent", fontWeight: 500, fontSize: theme.font.sizeSm, color: theme.colors.textPrimary }}
                  onClick={() => togglePlan(planName)}>
                  <span style={{ fontSize: "10px", width: "14px", textAlign: "center", color: theme.colors.textMuted }}>
                    {isExpanded ? "\u25BC" : "\u25B6"}
                  </span>
                  <span>{"\uD83D\uDCC1"}</span>
                  <span style={{ flex: 1 }}>{planName}</span>
                  <Badge variant="info">{pts.length}</Badge>
                </div>

                {/* Backup points (when expanded) */}
                {isExpanded && (
                  <div style={{ marginLeft: "8px", borderLeft: "2px solid " + theme.colors.panelBorder }}>
                    {pts.map((pt) => (
                      <div key={pt.backupId}
                        style={{ display: "flex", alignItems: "center", gap: "6px", padding: "4px 10px 4px 16px", cursor: "pointer", borderRadius: theme.radius.sm, margin: "2px 4px", background: selectedPoint?.backupId === pt.backupId ? theme.colors.primaryDim : "transparent", fontSize: theme.font.sizeXs, color: theme.colors.textSecondary }}
                        onClick={() => handleSelectPoint(pt)}>
                        <span>{"\uD83D\uDCC5"}</span>
                        <span style={{ flex: 1 }}>{pt.timestampLabel}</span>
                        <Badge variant="info">Full Backup</Badge>
                        {/* Delete button */}
                        <button onClick={(e: any) => { e.stopPropagation(); setShowConfirmDelete(pt.backupId); }}
                          title="Permanently delete this backup set"
                          style={{ background: "none", border: "none", cursor: "pointer", color: theme.colors.textMuted, padding: "2px 4px", opacity: 0.5, borderRadius: theme.radius.sm, transition: theme.transition.fast }}
                          onMouseEnter={(e: any) => { (e.currentTarget as HTMLElement).style.opacity = "1"; (e.currentTarget as HTMLElement).style.color = theme.colors.error; }}
                          onMouseLeave={(e: any) => { (e.currentTarget as HTMLElement).style.opacity = "0.5"; (e.currentTarget as HTMLElement).style.color = theme.colors.textMuted; }}>
                          <TrashIcon />
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Right Column: Selected point detail */}
      <div style={{ flex: 1, background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        {selectedPoint ? (
          <>
            <div style={{ padding: "16px", borderBottom: "1px solid " + theme.colors.panelBorder }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                <div>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <Button variant="secondary" onClick={handleBackToList} icon={<ArrowLeftIcon />}>Back</Button>
                    <h3 style={{ margin: 0, fontSize: theme.font.sizeLg, fontWeight: 600, color: theme.colors.textPrimary }}>{selectedPoint.jobName || "Restore Point"}</h3>
                    <Badge variant={selectedPoint.statusColor}>{selectedPoint.statusLabel}</Badge>
                  </div>
                  <p style={{ margin: "6px 0 0", fontSize: theme.font.sizeSm, color: theme.colors.textMuted }}>{selectedPoint.sourceRoot}</p>
                  <p style={{ margin: "2px 0 0", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
                    {"\uD83D\uDCC5"} {selectedPoint.timestampLabel}  {"\uD83D\uDCC4"} {selectedPoint.fileCount.toLocaleString()} files  {"\uD83D\uDCBE"} {selectedPoint.totalBytes}  Full Backup
                  </p>
                </div>
              </div>
            </div>

            <div style={{ flex: 1, overflowY: "auto", padding: "16px" }}>
              {restoreResult ? (
                <RestoreResultView result={restoreResult} onDone={handleBackToList} />
              ) : previewLoading ? (
                <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                  <div style={{ width: "200px", height: "20px", borderRadius: theme.radius.sm, background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" }} />
                  <SkeletonRestoreCard />
                </div>
              ) : previewError ? (
                <ErrorState title="Failed to load restore preview" message={previewError} onRetry={() => handleSelectPoint(selectedPoint)} />
              ) : preview ? (
                <>
                  <div style={{ marginBottom: "12px" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
                      <span style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
                        {"Files to restore (" + preview.total_files + " files, " + formatBytes(preview.total_bytes) + ")"}
                      </span>
                    </div>
                    <BackupTreeView files={preview.files} selectedPath={selectedTreePath} onSelect={setSelectedTreePath} />
                  </div>
                  <RestoreForm sourceRoot={selectedPoint.sourceRoot} onRestore={handleRestore} running={restoreRunning} />
                </>
              ) : null}
            </div>
          </>
        ) : (
          <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", flexDirection: "column", gap: "12px", color: theme.colors.textMuted }}>
            <ShieldIcon />
            <span style={{ fontSize: theme.font.sizeLg, fontWeight: 500, color: theme.colors.textSecondary }}>Select a backup point</span>
            <span style={{ fontSize: theme.font.sizeSm }}>Choose a restore point from the left panel to browse its contents and start a restore.</span>
          </div>
        )}
      </div>

      {/* Delete confirmation */}
      {showConfirmDelete && (
        <ConfirmDeleteDialog
          
          timestamp={points.find((p) => p.backupId === showConfirmDelete)?.timestampLabel ?? ""}
          onConfirm={handleDelete}
          onCancel={() => setShowConfirmDelete(null)}
          deleting={deleting}
        />
      )}

      {/* Toast */}
      {toastMessage && (<Toast message={toastMessage} onClose={() => setToastMessage(null)} />)}
    </div>
  );
}
