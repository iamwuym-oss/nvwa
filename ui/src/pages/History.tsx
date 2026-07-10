// ============================================================================
// History.tsx -- Operation History page
//
// Displays backup, restore, and verify operations in a filterable table.
//
// Data flow:
//   useEffect -> queryHistory(filter) -> HistoryRecordFormatted[] -> render
//
// States handled:
//   - Loading: skeleton rows
//   - Error:   error card with retry
//   - Empty:   guidance screen when no history exists
//   - Data:    table of history records with operation badge, status, detail
//   - Detail:  expanded row showing full operation details
// ============================================================================

import { useEffect, useState, useCallback } from "react";
import Badge from "../components/common/Badge";
import Button from "../components/common/Button";
import EmptyState from "../components/feedback/EmptyState";
import ErrorState from "../components/feedback/ErrorState";
import { theme } from "../theme";
import {
  queryHistory,
  listHistoryOperationTypes,
  HistoryRecordFormatted,
  HistoryFilter,
} from "../api/historyApi";

// ---------------------------------------------------------------------------
// Inline SVG icons
// ---------------------------------------------------------------------------

function RefreshIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="23 4 23 10 17 10"/>
      <path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/>
    </svg>
  );
}

function FilterIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
    </svg>
  );
}

function ChevronDownIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="6 9 12 15 18 9"/>
    </svg>
  );
}

function ChevronRightIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="9 18 15 12 9 6"/>
    </svg>
  );
}

// ---------------------------------------------------------------------------
// Operation badge color mapping
// ---------------------------------------------------------------------------

function operationBadgeVariant(op: string): "success" | "warning" | "error" | "info" | "neutral" {
  switch (op) {
    case "backup":  return "success";
    case "restore": return "info";
    case "verify":  return "warning";
    case "delete_backup_set": return "error";
    default:        return "neutral";
  }
}

// ---------------------------------------------------------------------------
// Skeleton row for loading state
// ---------------------------------------------------------------------------

function SkeletonRow() {
  const sk = {
    height: "16px",
    borderRadius: theme.radius.sm,
    background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)",
    backgroundSize: "200% 100%",
    animation: "skeletonPulse 1.5s infinite",
  };
  return (
    <tr>
      {[0, 1, 2, 3, 4, 5, 6].map((i) => (
        <td key={i} style={{ padding: "14px 12px" }}>
          <div style={{ width: i === 3 ? "100px" : "80px", ...sk }} />
        </td>
      ))}
    </tr>
  );
}

// ---------------------------------------------------------------------------
// Filter bar component
// ---------------------------------------------------------------------------

function FilterBar({
  operationTypes,
  selectedOp,
  onSelectOp,
}: {
  operationTypes: string[];
  selectedOp: string | null;
  onSelectOp: (op: string | null) => void;
}) {
  const btnStyle = (active: boolean): React.CSSProperties => ({
    padding: "6px 14px",
    border: "1px solid " + (active ? theme.colors.primary : theme.colors.panelBorder),
    borderRadius: theme.radius.full,
    background: active ? theme.colors.primaryDim : "transparent",
    color: active ? theme.colors.primary : theme.colors.textSecondary,
    fontSize: theme.font.sizeSm,
    fontWeight: active ? 600 : 400,
    cursor: "pointer",
    transition: theme.transition.fast,
    whiteSpace: "nowrap",
  });

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "4px", color: theme.colors.textMuted, fontSize: theme.font.sizeSm, marginRight: "4px" }}>
        <FilterIcon />
        <span>Filter:</span>
      </div>
      <button style={btnStyle(selectedOp === null)} onClick={() => onSelectOp(null)}>All</button>
      {operationTypes.map((op) => (
        <button key={op} style={btnStyle(selectedOp === op)} onClick={() => onSelectOp(op)}>
          {op.charAt(0).toUpperCase() + op.slice(1)}
        </button>
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Detail row component
// ---------------------------------------------------------------------------

function DetailRow({ record }: { record: HistoryRecordFormatted }) {
  const fieldStyle: React.CSSProperties = {
    display: "flex",
    justifyContent: "space-between",
    padding: "4px 0",
    fontSize: theme.font.sizeSm,
    borderBottom: "1px solid " + theme.colors.divider,
  };
  const labelStyle: React.CSSProperties = {
    color: theme.colors.textMuted,
    minWidth: "120px",
  };
  const valueStyle: React.CSSProperties = {
    color: theme.colors.textPrimary,
    textAlign: "right" as const,
    wordBreak: "break-all" as const,
  };

  return (
    <tr>
      <td colSpan={7} style={{ padding: 0 }}>
        <div style={{
          background: "rgba(0,0,0,0.15)",
          borderRadius: theme.radius.md,
          margin: "4px 0",
          padding: theme.spacing.md,
          border: "1px solid " + theme.colors.panelBorder,
        }}>
          <div style={{
            display: "grid",
            gridTemplateColumns: "1fr 1fr",
            gap: theme.spacing.sm,
          }}>
            <div style={fieldStyle}>
              <span style={labelStyle}>Backup ID</span>
              <span style={valueStyle}>{record.backupId}</span>
            </div>
            <div style={fieldStyle}>
              <span style={labelStyle}>Source Path</span>
              <span style={{ ...valueStyle, fontFamily: theme.font.mono, fontSize: theme.font.sizeXs }}>{record.sourceRoot}</span>
            </div>
            <div style={fieldStyle}>
              <span style={labelStyle}>Destination</span>
              <span style={{ ...valueStyle, fontFamily: theme.font.mono, fontSize: theme.font.sizeXs }}>{record.destPath}</span>
            </div>
            <div style={fieldStyle}>
              <span style={labelStyle}>Job Name</span>
              <span style={valueStyle}>{record.jobName ?? "N/A"}</span>
            </div>
            <div style={fieldStyle}>
              <span style={labelStyle}>Duration</span>
              <span style={valueStyle}>{record.durationFormatted}</span>
            </div>
            <div style={fieldStyle}>
              <span style={labelStyle}>Exit Info</span>
              <span style={valueStyle}>{record.exitInfo}</span>
            </div>
          </div>
        </div>
      </td>
    </tr>
  );
}

// ---------------------------------------------------------------------------
// History table
// ---------------------------------------------------------------------------

function HistoryTable({
  records,
  expandedId,
  onToggleExpand,
}: {
  records: HistoryRecordFormatted[];
  expandedId: string | null;
  onToggleExpand: (id: string) => void;
}) {
  const cellStyle: React.CSSProperties = {
    padding: "12px",
    fontSize: theme.font.sizeSm,
    color: theme.colors.textPrimary,
    borderBottom: "1px solid " + theme.colors.divider,
    whiteSpace: "nowrap",
  };

  const headerCellStyle: React.CSSProperties = {
    ...cellStyle,
    color: theme.colors.textMuted,
    fontWeight: 600,
    fontSize: theme.font.sizeXs,
    textTransform: "uppercase" as const,
    letterSpacing: "0.5px",
    borderBottom: "1px solid " + theme.colors.panelBorder,
    background: "rgba(0,0,0,0.15)",
  };

  return (
    <div style={{ overflowX: "auto", borderRadius: theme.radius.lg, border: "1px solid " + theme.colors.panelBorder }}>
      <table style={{ width: "100%", borderCollapse: "collapse", minWidth: "800px" }}>
        <thead>
          <tr>
            <th style={{ ...headerCellStyle, width: "32px" }}>{/* expand */}</th>
            <th style={headerCellStyle}>Time</th>
            <th style={headerCellStyle}>Operation</th>
            <th style={headerCellStyle}>Job</th>
            <th style={headerCellStyle}>Status</th>
            <th style={headerCellStyle}>Files</th>
            <th style={{ ...headerCellStyle, textAlign: "right" as const }}>Size</th>
          </tr>
        </thead>
        <tbody>
          {records.map((r) => {
            const isExpanded = expandedId === r.backupId;
            return (
              <>
                <tr
                  key={r.backupId}
                  onClick={() => onToggleExpand(r.backupId)}
                  style={{
                    cursor: "pointer",
                    transition: theme.transition.fast,
                    background: isExpanded ? "rgba(22,131,255,0.04)" : undefined,
                  }}
                  onMouseEnter={(e) => { if (!isExpanded) e.currentTarget.style.background = "rgba(255,255,255,0.02)"; }}
                  onMouseLeave={(e) => { if (!isExpanded) e.currentTarget.style.background = "transparent"; }}
                >
                  <td style={{ ...cellStyle, textAlign: "center" as const }}>
                    {isExpanded ? <ChevronDownIcon /> : <ChevronRightIcon />}
                  </td>
                  <td style={cellStyle}>
                    <div style={{ color: theme.colors.textPrimary }}>{r.timestampLabel}</div>
                    <div style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, marginTop: "2px" }}>
                      {new Date(r.timestamp).toLocaleString()}
                    </div>
                  </td>
                  <td style={cellStyle}>
                    <Badge variant={operationBadgeVariant(r.operation)}>{r.operationLabel}</Badge>
                  </td>
                  <td style={{ ...cellStyle, color: r.jobName ? theme.colors.textPrimary : theme.colors.textMuted }}>
                    {r.jobName ?? "N/A"}
                  </td>
                  <td style={cellStyle}>
                    <Badge variant={r.statusColor}>{r.statusLabel}</Badge>
                  </td>
                  <td style={cellStyle}>{r.fileCount.toLocaleString()}</td>
                  <td style={{ ...cellStyle, textAlign: "right" as const, fontFamily: theme.font.mono }}>
                    {r.totalBytesFormatted}
                  </td>
                </tr>
                {isExpanded && <DetailRow record={r} />}
              </>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main History Page
// ---------------------------------------------------------------------------

export default function History() {
  const [records, setRecords] = useState<HistoryRecordFormatted[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedOp, setSelectedOp] = useState<string | null>(null);
  const [operationTypes, setOperationTypes] = useState<string[]>(["backup", "restore", "verify"]);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  // 加载可用的操作类型
  useEffect(() => {
    listHistoryOperationTypes()
      .then(setOperationTypes)
      .catch(() => { /* keep defaults */ });
  }, []);

  // 加载历史记录
  const fetchHistory = useCallback(() => {
    setLoading(true);
    setError(null);

    const filter: HistoryFilter = {};
    if (selectedOp) {
      filter.operation = selectedOp;
    }
    filter.limit = 100;

    queryHistory(filter)
      .then((result) => {
        setRecords(result.records);
        setTotal(result.total);
        setLoading(false);
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : "Failed to load backup history";
        setError(msg);
        setLoading(false);
      });
  }, [selectedOp]);

  useEffect(() => {
    fetchHistory();
  }, [fetchHistory]);

  // 切换过滤器时重置展开状态
  useEffect(() => {
    setExpandedId(null);
  }, [selectedOp]);

  const handleToggleExpand = useCallback((id: string) => {
    setExpandedId((prev) => prev === id ? null : id);
  }, []);

  // ---- Loading ----
  if (loading) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <div style={{ marginBottom: "24px" }}>
          <div style={{ width: "200px", height: "28px", borderRadius: theme.radius.sm, background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" }} />
        </div>
        <FilterBar operationTypes={operationTypes} selectedOp={null} onSelectOp={() => {}} />
        <div style={{ marginTop: "20px" }}>
          <div style={{ overflowX: "auto", borderRadius: theme.radius.lg, border: "1px solid " + theme.colors.panelBorder }}>
            <table style={{ width: "100%", borderCollapse: "collapse", minWidth: "800px" }}>
              <thead>
                <tr>
                  {["", "Time", "Operation", "Job", "Status", "Files", "Size"].map((h) => (
                    <th key={h} style={{ padding: "12px", fontSize: theme.font.sizeXs, color: theme.colors.textMuted, fontWeight: 600, borderBottom: "1px solid " + theme.colors.panelBorder, background: "rgba(0,0,0,0.15)" }}>{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {[0, 1, 2, 3, 4].map((i) => <SkeletonRow key={i} />)}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    );
  }

  // ---- Error ----
  if (error) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <ErrorState title="Failed to load backup history" message={error} onRetry={fetchHistory} />
      </div>
    );
  }

  // ---- Empty ----
  if (records.length === 0) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <div style={{ marginBottom: "24px" }}>
          <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>History</h2>
          <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>
            Backup and restore operation history
          </p>
        </div>
        <FilterBar operationTypes={operationTypes} selectedOp={selectedOp} onSelectOp={setSelectedOp} />
        <div style={{ marginTop: "20px" }}>
          <EmptyState
            title="No History Records"
            description={selectedOp
              ? "No " + selectedOp + " operations found. Try a different filter."
              : "Run a backup or restore operation to see it here."}
          />
        </div>
      </div>
    );
  }

  // ---- Data ----
  const opCount = selectedOp
    ? records.filter((r) => r.operation === selectedOp).length
    : total;

  return (
    <div style={{ padding: theme.spacing.lg }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "24px" }}>
        <div>
          <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>History</h2>
          <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>
            {total} operation{total !== 1 ? "s" : ""} recorded
            {selectedOp ? " (" + opCount + " " + selectedOp + ")" : ""}
          </p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <Button variant="secondary" size="sm" onClick={fetchHistory} icon={<RefreshIcon />}>Refresh</Button>
        </div>
      </div>

      <FilterBar operationTypes={operationTypes} selectedOp={selectedOp} onSelectOp={setSelectedOp} />

      <div style={{ marginTop: "16px" }}>
        <HistoryTable
          records={records}
          expandedId={expandedId}
          onToggleExpand={handleToggleExpand}
        />
      </div>
    </div>
  );
}
