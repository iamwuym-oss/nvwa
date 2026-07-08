// ============================================================================
// Restore.tsx -- Restore management page
//
// Data flow:
//   useEffect -> listRestorePoints() -> RestorePointViewFormatted[] -> render
//   On point select -> getRestorePreview(backupId) -> RestorePreview -> render
//   On restore -> executeRestore(request) -> RestoreOperationResult -> render
//
// States handled:
//   - Loading: skeleton cards for restore point list
//   - Error:   error card with reasons + retry button
//   - Empty:   guidance screen when no restore points are available
//   - List:    restore point list with selection
//   - Preview: file list for selected backup point + restore form
//   - Running: restore in progress with spinner
//   - Result:  restore completed with success/failure summary
// ============================================================================

import { useEffect, useState, useCallback, useMemo } from "react";
import Badge from "../components/common/Badge";
import Button from "../components/common/Button";import PathInput from "../components/common/PathInput";
import BackupTreeView from "../components/common/BackupTreeView";
import EmptyState from "../components/feedback/EmptyState";
import ErrorState from "../components/feedback/ErrorState";
import { theme } from "../theme";
import {
  listRestorePoints,
  getRestorePreview,
  executeRestore,
  RestorePointViewFormatted,
  RestorePreview,
  RestoreOperationResult,
} from "../api/restoreApi";

// ---------------------------------------------------------------------------
// Inline SVG icons
// ---------------------------------------------------------------------------

function RestoreIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="1 4 1 10 7 10"/>
      <path d="M3.51 15a9 9 0 102.13-9.36L1 10"/>
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


function ShieldIcon() {
  return (
    <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
    </svg>
  );
}

// ---------------------------------------------------------------------------
// Skeleton card for loading state
// ---------------------------------------------------------------------------

function SkeletonRestoreCard() {
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
        <div style={{ width: "160px", ...sk }} />
        <div style={{ width: "70px", ...sk, height: "22px", borderRadius: theme.radius.full }} />
      </div>
      <div style={{ display: "flex", gap: "24px", marginBottom: "12px" }}>
        <div style={{ width: "180px", ...sk }} />
        <div style={{ width: "120px", ...sk }} />
      </div>
      <div style={{ width: "280px", ...sk }} />
    </div>
  );
}


// ---------------------------------------------------------------------------
// Restore Form
// ---------------------------------------------------------------------------

interface RestoreFormProps {
  backupId: string;
  sourceRoot: string;
  onRestore: (request: { backupId: string; dest: string; overwrite: boolean }) => Promise<void>;
  running: boolean;
}

function RestoreForm({ backupId, sourceRoot, onRestore, running }: RestoreFormProps) {
  const [dest, setDest] = useState("");
  const [overwrite, setOverwrite] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async () => {
    if (!dest.trim()) {
      setError("Destination path is required");
      return;
    }
    setError(null);
    await onRestore({ backupId, dest: dest.trim(), overwrite });
  };

  const suggestedDest = sourceRoot.replace(/^[A-Z]:/, "D:\\Restore") || "D:\\Restore";

  return (
    <div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, padding: theme.spacing.lg }}>
      <div style={{ fontSize: theme.font.sizeMd, fontWeight: 700, color: theme.colors.textPrimary, marginBottom: "16px" }}>
        Restore Settings
      </div>

      {/* Destination path */}
      <div style={{ marginBottom: "16px" }}>
        <label style={{ display: "block", fontSize: theme.font.sizeXs, fontWeight: 600, color: theme.colors.textSecondary, marginBottom: "6px", textTransform: "uppercase", letterSpacing: "0.5px" }}>
          Destination Path
        </label>
        <div style={{ display: "flex", gap: "8px" }}>
          <div style={{ flex: 1 }}>
            <PathInput
              value={dest}
              onChange={(v) => { setDest(v); setError(null); }}
              placeholder={suggestedDest}
              disabled={running}
            />
          </div>
          <Button variant="ghost" onClick={() => setDest(suggestedDest)} disabled={running}>
            Use Default
          </Button>
        </div>
        {error && (
          <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.error, marginTop: "4px" }}>
            {error}
          </div>
        )}
      </div>

      {/* Overwrite toggle */}
      <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "20px" }}>
        <input
          type="checkbox"
          id="overwrite"
          checked={overwrite}
          onChange={(e) => setOverwrite(e.target.checked)}
          disabled={running}
          style={{ accentColor: theme.colors.primary, width: "16px", height: "16px", cursor: running ? "not-allowed" : "pointer" }}
        />
        <label htmlFor="overwrite" style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, cursor: "pointer" }}>
          Overwrite existing files
        </label>
      </div>

      {/* Restore button */}
      <Button
        variant="primary"
        size="md"
        onClick={handleSubmit}
        disabled={running}
        icon={running ? undefined : <RestoreIcon />}
      >
        {running ? "Restoring..." : "Start Restore"}
      </Button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Restore Result
// ---------------------------------------------------------------------------

interface RestoreResultViewProps {
  result: RestoreOperationResult;
  onDone: () => void;
}

function RestoreResultView({ result, onDone }: RestoreResultViewProps) {
  const isSuccess = result.status === "success";
  const icon = isSuccess ? <CheckCircleIcon /> : <XCircleIcon />;
  const statusColor = isSuccess ? theme.colors.success : theme.colors.error;
  const statusBg = isSuccess ? theme.colors.successDim : theme.colors.errorDim;
  const title = isSuccess ? "Restore Completed Successfully" : "Restore Failed";
  const durationSec = Math.round(result.duration_ms / 1000);

  return (
    <div style={{ background: statusBg, border: "1px solid " + statusColor + "30", borderRadius: theme.radius.xl, padding: theme.spacing.lg }}>
      <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "16px" }}>
        {icon}
        <div>
          <div style={{ fontSize: theme.font.sizeMd, fontWeight: 700, color: theme.colors.textPrimary }}>{title}</div>
          <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>{result.restore_id}</div>
        </div>
      </div>
      <div style={{ display: "flex", gap: "24px", flexWrap: "wrap", marginBottom: "4px" }}>
        <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
          <span style={{ color: theme.colors.textMuted }}>Restored:</span> {result.restored_count.toLocaleString()} files
        </div>
        <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
          <span style={{ color: theme.colors.textMuted }}>Skipped:</span> {result.skipped_count.toLocaleString()}
        </div>
        {result.checksum_failures > 0 && (
          <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.warning }}>
            <span style={{ color: theme.colors.textMuted }}>Checksum failures:</span> {result.checksum_failures}
          </div>
        )}
        <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
          <span style={{ color: theme.colors.textMuted }}>Duration:</span> {durationSec}s
        </div>
      </div>
      {result.error && (
        <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.error, marginTop: "8px", padding: "8px 12px", background: "rgba(0,0,0,0.2)", borderRadius: theme.radius.md }}>
          {result.error}
        </div>
      )}
      <div style={{ marginTop: "16px" }}>
        <Button variant="secondary" size="md" icon={<ArrowLeftIcon />} onClick={onDone}>
          Back to restore points
        </Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main Restore Page
// ---------------------------------------------------------------------------



// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return val.toFixed(1) + " " + units[i];
}

// ---------------------------------------------------------------------------
// Main Restore Page
// ---------------------------------------------------------------------------

export default function Restore() {
  const [points, setPoints] = useState<RestorePointViewFormatted[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Preview state
  const [selectedPoint, setSelectedPoint] = useState<RestorePointViewFormatted | null>(null);
  const [preview, setPreview] = useState<RestorePreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  // Restore state
  const [restoreRunning, setRestoreRunning] = useState(false);
  const [restoreResult, setRestoreResult] = useState<RestoreOperationResult | null>(null);

  // Two-column layout state
  const [expandedPlans, setExpandedPlans] = useState<Set<string>>(new Set());
  const [selectedTreePath, setSelectedTreePath] = useState<string | null>(null);

  // Group points by jobName
  const groupedPoints = useMemo(() => {
    const groups = new Map<string, RestorePointViewFormatted[]>();
    for (const p of points) {
      const key = p.jobName || "Unnamed Plan";
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)!.push(p);
    }
    for (const [, pts] of groups) {
      pts.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
    }
    return groups;
  }, [points]);

  const totalPointCount = points.length;
  const totalPlanCount = groupedPoints.size;

  const togglePlan = (name: string) => {
    setExpandedPlans((prev) => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  };

  const fetchPoints = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listRestorePoints();
      setPoints(data);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load restore points";
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { fetchPoints(); }, [fetchPoints]);

  const handleSelectPoint = useCallback(async (point: RestorePointViewFormatted) => {
    setSelectedPoint(point);
    setPreview(null);
    setPreviewLoading(true);
    setPreviewError(null);
    setRestoreResult(null);
    setSelectedTreePath(null);
    try {
      const data = await getRestorePreview(point.backupId);
      setPreview(data);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load preview";
      setPreviewError(msg);
    } finally {
      setPreviewLoading(false);
    }
  }, []);

  const handleBackToList = useCallback(() => {
    setSelectedPoint(null);
    setPreview(null);
    setPreviewError(null);
    setRestoreResult(null);
    setSelectedTreePath(null);
  }, []);

  const handleRestore = useCallback(async (request: { backupId: string; dest: string; overwrite: boolean }) => {
    setRestoreRunning(true);
    setRestoreResult(null);
    try {
      const res = await executeRestore({
        backup_id: request.backupId,
        dest: request.dest,
        overwrite: request.overwrite,
      });
      setRestoreResult(res);
    } catch (err: unknown) {
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
    } finally {
      setRestoreRunning(false);
    }
  }, []);

  // --- Loading ---
  if (loading) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <div style={{ marginBottom: "24px" }}>
          <div style={{ width: "200px", height: "28px", borderRadius: theme.radius.sm, background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" }} />
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <SkeletonRestoreCard /><SkeletonRestoreCard /><SkeletonRestoreCard />
        </div>
      </div>
    );
  }

  // --- Error ---
  if (error) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <ErrorState
          title="Failed to load restore points"
          message={error}
          reasons={[
            "Backup configuration may be missing",
            "Storage location may be unavailable",
            "No backup history found",
          ]}
          onRetry={fetchPoints}
        />
      </div>
    );
  }

  // --- Empty ---
  if (points.length === 0) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <EmptyState
          icon={<ShieldIcon />}
          title="No Restore Points Available"
          description="Run a backup first to create restore points. Once backups exist, you can restore your files from this page."
        />
      </div>
    );
  }

  // --- Two-column layout ---
  return (
    <div style={{ display: "flex", gap: "16px", padding: theme.spacing.lg, height: "calc(100vh - 100px)" }}>

      {/* Left Column: Restore Points by Plan */}
      <div style={{
        width: "320px",
        minWidth: "320px",
        background: theme.colors.panel,
        border: "1px solid " + theme.colors.panelBorder,
        borderRadius: theme.radius.lg,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}>
        {/* Header */}
        <div style={{ padding: "16px", borderBottom: "1px solid " + theme.colors.panelBorder }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <h3 style={{ margin: 0, fontSize: theme.font.sizeLg, fontWeight: 600, color: theme.colors.textPrimary }}>
              Restore Points
            </h3>
            <Button variant="secondary" onClick={fetchPoints} icon={<RefreshIcon />}>Refresh</Button>
          </div>
          <p style={{ margin: "4px 0 0", fontSize: theme.font.sizeSm, color: theme.colors.textMuted }}>
            {totalPointCount} point{totalPointCount !== 1 ? "s" : ""} in {totalPlanCount} plan{totalPlanCount !== 1 ? "s" : ""}
          </p>
        </div>

        {/* Scrollable plan list */}
        <div style={{ flex: 1, overflowY: "auto", padding: "8px" }}>
          {Array.from(groupedPoints.entries()).map(([planName, pts]) => {
            const isExpanded = expandedPlans.has(planName);
            return (
              <div key={planName} style={{ marginBottom: "4px" }}>
                {/* Plan header */}
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "8px",
                    padding: "8px 10px",
                    borderRadius: theme.radius.sm,
                    cursor: "pointer",
                    background: isExpanded ? theme.colors.primaryDim : "transparent",
                    fontWeight: 500,
                    fontSize: theme.font.sizeSm,
                    color: theme.colors.textPrimary,
                  }}
                  onClick={() => togglePlan(planName)}
                >
                  <span style={{ fontSize: "10px", width: "14px", textAlign: "center", color: theme.colors.textMuted }}>
                    {isExpanded ? String.fromCharCode(9660) : String.fromCharCode(9654)}
                  </span>
                  <span>{String.fromCharCode(128193)}</span>
                  <span style={{ flex: 1 }}>{planName}</span>
                  <Badge variant="info">{pts.length}</Badge>
                </div>

                {/* Backup points (when expanded) */}
                {isExpanded && (
                  <div style={{ marginLeft: "8px", borderLeft: "2px solid " + theme.colors.panelBorder }}>
                    {pts.map((pt) => (
                      <div
                        key={pt.backupId}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: "8px",
                          padding: "6px 10px 6px 16px",
                          cursor: "pointer",
                          borderRadius: theme.radius.sm,
                          margin: "2px 4px",
                          background: selectedPoint?.backupId === pt.backupId ? theme.colors.primaryDim : "transparent",
                          fontSize: theme.font.sizeXs,
                          color: theme.colors.textSecondary,
                        }}
                        onClick={() => handleSelectPoint(pt)}
                      >
                        <span>{String.fromCharCode(128197)}</span>
                        <span style={{ flex: 1 }}>{pt.timestampLabel}</span>
                        <Badge variant="info">Full Backup</Badge>
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
      <div style={{
        flex: 1,
        background: theme.colors.panel,
        border: "1px solid " + theme.colors.panelBorder,
        borderRadius: theme.radius.lg,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}>
        {selectedPoint ? (
          <>
            {/* Point summary header */}
            <div style={{ padding: "16px", borderBottom: "1px solid " + theme.colors.panelBorder }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                <div>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <Button variant="secondary" onClick={handleBackToList} icon={<ArrowLeftIcon />}>Back</Button>
                    <h3 style={{ margin: 0, fontSize: theme.font.sizeLg, fontWeight: 600, color: theme.colors.textPrimary }}>
                      {selectedPoint.jobName || "Restore Point"}
                    </h3>
                    <Badge variant={selectedPoint.statusColor}>{selectedPoint.statusLabel}</Badge>
                  </div>
                  <p style={{ margin: "6px 0 0", fontSize: theme.font.sizeSm, color: theme.colors.textMuted }}>
                    {selectedPoint.sourceRoot}
                  </p>
                  <p style={{ margin: "2px 0 0", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
                    {String.fromCharCode(128197)} {selectedPoint.timestampLabel}  {String.fromCharCode(128196)} {selectedPoint.fileCount.toLocaleString()} files  {String.fromCharCode(128190)} {selectedPoint.totalBytes}  Full Backup
                  </p>
                </div>
              </div>
            </div>

            {/* Content area: tree + restore */}
            <div style={{ flex: 1, overflowY: "auto", padding: "16px" }}>
              {restoreResult ? (
                <RestoreResultView result={restoreResult} onDone={handleBackToList} />
              ) : previewLoading ? (
                <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                  <div style={{ width: "200px", height: "20px", borderRadius: theme.radius.sm, background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" }} />
                  <SkeletonRestoreCard />
                </div>
              ) : previewError ? (
                <ErrorState
                  title="Failed to load restore preview"
                  message={previewError}
                  onRetry={() => handleSelectPoint(selectedPoint)}
                />
              ) : preview ? (
                <>
                  {/* File tree */}
                  <div style={{ marginBottom: "12px" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
                      <span style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
                        {"Files to restore (" + preview.total_files + " files, " + formatBytes(preview.total_bytes) + ")"}
                      </span>
                    </div>
                    <BackupTreeView
                      files={preview.files}
                      selectedPath={selectedTreePath}
                      onSelect={setSelectedTreePath}
                    />
                  </div>
                  {/* Restore form */}
                  <RestoreForm
                    backupId={selectedPoint.backupId}
                    sourceRoot={selectedPoint.sourceRoot}
                    onRestore={handleRestore}
                    running={restoreRunning}
                  />
                </>
              ) : null}
            </div>
          </>
        ) : (
          /* Empty detail state */
          <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", flexDirection: "column", gap: "12px", color: theme.colors.textMuted }}>
            <ShieldIcon />
            <span style={{ fontSize: theme.font.sizeLg, fontWeight: 500, color: theme.colors.textSecondary }}>
              Select a backup point
            </span>
            <span style={{ fontSize: theme.font.sizeSm }}>
              Choose a restore point from the left panel to browse its contents and start a restore.
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
