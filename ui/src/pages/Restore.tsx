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

import { useEffect, useState, useCallback } from "react";
import Badge from "../components/common/Badge";
import Button from "../components/common/Button";
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
// Restore Point Card
// ---------------------------------------------------------------------------

interface RestorePointCardProps {
  point: RestorePointViewFormatted;
  onSelect: () => void;
}

function RestorePointCard({ point, onSelect }: RestorePointCardProps) {
  return (
    <div
      onClick={onSelect}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onSelect(); } }}
      style={{
        background: theme.colors.panel,
        border: "1px solid " + theme.colors.panelBorder,
        borderRadius: theme.radius.lg,
        padding: theme.spacing.lg,
        cursor: "pointer",
        transition: theme.transition.fast,
      }}
      onMouseEnter={(e) => { e.currentTarget.style.borderColor = theme.colors.primary + "40"; e.currentTarget.style.background = theme.colors.panelHover; }}
      onMouseLeave={(e) => { e.currentTarget.style.borderColor = theme.colors.panelBorder; e.currentTarget.style.background = theme.colors.panel; }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "12px" }}>
        <div>
          <div style={{ fontSize: theme.font.sizeMd, fontWeight: 700, color: theme.colors.textPrimary, marginBottom: "2px" }}>
            {point.jobName ?? "Unknown Job"}
          </div>
          <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
            Backup ID: {point.backupId}
          </div>
        </div>
        <Badge variant={point.statusColor}>{point.statusLabel}</Badge>
      </div>
      <div style={{ display: "flex", gap: "16px", flexWrap: "wrap", marginBottom: "8px" }}>
        <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
          <span style={{ color: theme.colors.textMuted }}>Files:</span> {point.fileCount.toLocaleString()}
        </div>
        <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
          <span style={{ color: theme.colors.textMuted }}>Size:</span> {point.totalBytes}
        </div>
        <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
          <span style={{ color: theme.colors.textMuted }}>Source:</span> {point.sourceRoot}
        </div>
      </div>
      <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
        {point.timestampLabel} &middot; {point.timestamp}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Preview View
// ---------------------------------------------------------------------------

interface PreviewViewProps {
  preview: RestorePreview;
  onBack: () => void;
}

function PreviewView({ preview, onBack }: PreviewViewProps) {
  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "20px" }}>
        <Button variant="ghost" size="sm" onClick={onBack} icon={<ArrowLeftIcon />}>Back to restore points</Button>
      </div>

      {/* Point summary */}
      <div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, padding: theme.spacing.lg, marginBottom: "20px" }}>
        <div style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, marginBottom: "8px" }}>
          {preview.point.source_root}
        </div>
        <div style={{ display: "flex", gap: "24px", flexWrap: "wrap", marginBottom: "4px" }}>
          <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
            <span style={{ color: theme.colors.textMuted }}>Files:</span> {preview.total_files.toLocaleString()}
          </div>
          <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
            <span style={{ color: theme.colors.textMuted }}>Total size:</span> {
              preview.total_bytes > 0
                ? (() => {
                    const units = ["B", "KB", "MB", "GB", "TB"];
                    const i = Math.floor(Math.log(preview.total_bytes) / Math.log(1024));
                    const val = preview.total_bytes / Math.pow(1024, i);
                    return `${val.toFixed(1)} ${units[i]}`;
                  })()
                : "0 B"
            }
          </div>
          <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
            <span style={{ color: theme.colors.textMuted }}>Backup:</span> {preview.point.timestamp}
          </div>
        </div>
      </div>

      {/* File list */}
      <div style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg, padding: theme.spacing.lg }}>
        <div style={{ fontSize: theme.font.sizeSm, fontWeight: 600, color: theme.colors.textSecondary, textTransform: "uppercase", letterSpacing: "0.5px", marginBottom: "12px" }}>
          Files to restore ({preview.files.length.toLocaleString()} files)
        </div>
        {preview.files.length === 0 ? (
          <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, padding: "20px 0", textAlign: "center" }}>
            No files to restore in this backup point.
          </div>
        ) : preview.files.length <= 50 ? (
          <div style={{ maxHeight: "320px", overflowY: "auto" }}>
            {preview.files.map((file, idx) => (
              <div key={idx} style={{ display: "flex", justifyContent: "space-between", padding: "4px 0", borderBottom: idx < preview.files.length - 1 ? "1px solid " + theme.colors.divider : "none" }}>
                <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textPrimary, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1, marginRight: "12px" }}>
                  {file.relative_path}
                </div>
                <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, whiteSpace: "nowrap" }}>
                  {(() => {
                    const units = ["B", "KB", "MB", "GB", "TB"];
                    const i = Math.floor(Math.log(file.size_bytes) / Math.log(1024));
                    const val = file.size_bytes / Math.pow(1024, i);
                    return `${val.toFixed(1)} ${units[i]}`;
                  })()}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, padding: "12px 0" }}>
            Showing first 50 of {preview.files.length.toLocaleString()} files. Full restore will include all files.
          </div>
        )}
      </div>
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
  const inputStyle: React.CSSProperties = {
    width: "100%",
    padding: "8px 12px",
    background: "rgba(0,0,0,0.3)",
    border: "1px solid " + theme.colors.panelBorder,
    borderRadius: theme.radius.md,
    color: theme.colors.textPrimary,
    fontSize: theme.font.sizeSm,
    fontFamily: theme.font.family,
    outline: "none",
    boxSizing: "border-box",
  };

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
          <input
            type="text"
            value={dest}
            onChange={(e) => { setDest(e.target.value); setError(null); }}
            placeholder={suggestedDest}
            disabled={running}
            style={inputStyle}
          />
          <Button variant="ghost" size="sm" onClick={() => setDest(suggestedDest)} disabled={running}>
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

  // Fetch restore points
  const fetchPoints = useCallback(() => {
    setLoading(true);
    setError(null);

    listRestorePoints()
      .then((pts) => {
        setPoints(pts);
        setLoading(false);
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : "Failed to load restore points";
        setError(msg);
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    fetchPoints();
  }, [fetchPoints]);

  // Select a restore point and load preview
  const handleSelectPoint = useCallback(async (point: RestorePointViewFormatted) => {
    setSelectedPoint(point);
    setPreviewLoading(true);
    setPreviewError(null);
    setPreview(null);
    setRestoreResult(null);

    try {
      const p = await getRestorePreview(point.backupId);
      setPreview(p);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to load preview";
      setPreviewError(msg);
    } finally {
      setPreviewLoading(false);
    }
  }, []);

  // Go back to restore point list
  const handleBackToList = useCallback(() => {
    setSelectedPoint(null);
    setPreview(null);
    setPreviewError(null);
    setRestoreResult(null);
  }, []);

  // Execute restore
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

  // ---- Loading ----
  if (loading) {
    return (
      <div style={{ padding: theme.spacing.lg, maxWidth: "900px" }}>
        <div style={{ marginBottom: "24px" }}>
          <div style={{ width: "200px", height: "28px", borderRadius: theme.radius.sm, ...{ background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" } }} />
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <SkeletonRestoreCard /><SkeletonRestoreCard /><SkeletonRestoreCard />
        </div>
      </div>
    );
  }

  // ---- Error ----
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

  // ---- Empty ----
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

  // ---- Preview / Restore flow ----
  if (selectedPoint) {
    return (
      <div style={{ padding: theme.spacing.lg, maxWidth: "900px" }}>
        {/* Restore result */}
        {restoreResult && (
          <div style={{ marginBottom: "20px" }}>
            <RestoreResultView result={restoreResult} onDone={handleBackToList} />
          </div>
        )}

        {/* Preview section */}
        {!restoreResult && (
          <>
            {previewLoading ? (
              <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                <div style={{ width: "200px", height: "20px", borderRadius: theme.radius.sm, ...{ background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" } }} />
                <SkeletonRestoreCard />
              </div>
            ) : previewError ? (
              <ErrorState
                title="Failed to load restore preview"
                message={previewError}
                onRetry={() => handleSelectPoint(selectedPoint)}
              />
            ) : preview ? (
              <PreviewView preview={preview} onBack={handleBackToList} />
            ) : null}
          </>
        )}

        {/* Restore form (shown after preview loads, or after result dismissed) */}
        {preview && !restoreResult && (
          <div style={{ marginTop: "20px" }}>
            <RestoreForm
              backupId={selectedPoint.backupId}
              sourceRoot={selectedPoint.sourceRoot}
              onRestore={handleRestore}
              running={restoreRunning}
            />
          </div>
        )}
      </div>
    );
  }

  // ---- List View ----
  return (
    <div style={{ padding: theme.spacing.lg, maxWidth: "900px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "24px" }}>
        <div>
          <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>Restore</h2>
          <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>
            {points.length} restore point{points.length > 1 ? "s" : ""} available
          </p>
        </div>
        <Button variant="secondary" size="sm" onClick={fetchPoints} icon={<RefreshIcon />}>Refresh</Button>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {points.map((point) => (
          <RestorePointCard
            key={point.backupId}
            point={point}
            onSelect={() => handleSelectPoint(point)}
          />
        ))}
      </div>
    </div>
  );
}
