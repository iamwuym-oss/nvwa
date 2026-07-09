// ============================================================================
// historyApi.ts -- History page API bridge
//
// This file is the ONLY place where the History page talks to the backend.
// It provides:
//   - queryHistory(filter)  -- query operation history with optional filters
//   - listHistoryOperationTypes() -- available operation types
//
// The History.tsx page never imports mockData.ts or calls invoke() directly.
// ============================================================================

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// TypeScript interfaces matching Rust src/app/models/history.rs
// ---------------------------------------------------------------------------

export interface HistoryRecordView {
  backup_id: string;
  operation: string;
  timestamp: string;
  source_root: string;
  dest_path: string;
  job_name: string | null;
  file_count: number;
  total_bytes: number;
  duration_ms: number;
  status: string;
  exit_info: string;
}

export interface HistoryQueryResult {
  total: number;
  records: HistoryRecordView[];
}

export interface HistoryFilter {
  limit?: number | null;
  operation?: string | null;
}

// ---------------------------------------------------------------------------
// Formatted view model for UI consumption
// ---------------------------------------------------------------------------

export interface HistoryRecordFormatted {
  backupId: string;
  operation: string;
  operationLabel: string;
  operationColor: "success" | "warning" | "error" | "info" | "neutral";
  timestamp: string;
  timestampLabel: string;
  sourceRoot: string;
  destPath: string;
  jobName: string | null;
  fileCount: number;
  totalBytesFormatted: string;
  durationFormatted: string;
  status: string;
  statusLabel: string;
  statusColor: "success" | "warning" | "error" | "info" | "neutral";
  exitInfo: string;
}

// ---------------------------------------------------------------------------
// Formatters
// ---------------------------------------------------------------------------

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return val.toFixed(1) + " " + units[i];
}

function formatDuration(ms: number): string {
  if (ms < 1000) return ms + "ms";
  if (ms < 60000) return (ms / 1000).toFixed(1) + "s";
  const min = Math.floor(ms / 60000);
  const sec = Math.floor((ms % 60000) / 1000);
  return min + "m " + sec + "s";
}

function formatTimestamp(ts: string): string {
  try {
    const d = new Date(ts);
    if (isNaN(d.getTime())) return ts;
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffHours = Math.floor(diffMs / 3600000);
    if (diffHours < 1) return "Just now";
    if (diffHours < 24) return diffHours + "h ago";
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 7) return diffDays + "d ago";
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
  } catch {
    return ts;
  }
}

function operationDisplay(op: string): { label: string; color: "success" | "warning" | "error" | "info" | "neutral" } {
  switch (op) {
    case "backup":  return { label: "Backup",  color: "success" };
    case "restore": return { label: "Restore", color: "info" };
    case "verify":  return { label: "Verify",  color: "warning" };
    default:        return { label: op,        color: "neutral" };
  }
}

function statusDisplay(status: string): { label: string; color: "success" | "warning" | "error" | "info" | "neutral" } {
  switch (status) {
    case "success": return { label: "Success", color: "success" };
    case "failure": return { label: "Failed",  color: "error" };
    case "partial": return { label: "Partial", color: "warning" };
    default:        return { label: status,    color: "neutral" };
  }
}

function formatRecord(r: HistoryRecordView): HistoryRecordFormatted {
  const op = operationDisplay(r.operation);
  const st = statusDisplay(r.status);
  return {
    backupId: r.backup_id,
    operation: r.operation,
    operationLabel: op.label,
    operationColor: op.color,
    timestamp: r.timestamp,
    timestampLabel: formatTimestamp(r.timestamp),
    sourceRoot: r.source_root,
    destPath: r.dest_path,
    jobName: r.job_name,
    fileCount: r.file_count,
    totalBytesFormatted: formatBytes(r.total_bytes),
    durationFormatted: formatDuration(r.duration_ms),
    status: r.status,
    statusLabel: st.label,
    statusColor: st.color,
    exitInfo: r.exit_info,
  };
}

// ---------------------------------------------------------------------------
// Mock data for UI development
// ---------------------------------------------------------------------------

const MOCK_RECORDS: HistoryRecordView[] = [
  {
    backup_id: "bp-20260708-220000",
    operation: "backup",
    timestamp: "2026-07-08T22:00:00Z",
    source_root: "C:\\Users\\Tony\\Documents",
    dest_path: "D:\\Backups\\Documents",
    job_name: "Documents",
    file_count: 15234,
    total_bytes: 26306674688,
    duration_ms: 180000,
    status: "success",
    exit_info: "Completed successfully",
  },
  {
    backup_id: "bp-20260708-210000",
    operation: "backup",
    timestamp: "2026-07-08T21:00:00Z",
    source_root: "C:\\Users\\Tony\\Projects",
    dest_path: "D:\\Backups\\Projects",
    job_name: "Projects",
    file_count: 88720,
    total_bytes: 166994360320,
    duration_ms: 480000,
    status: "success",
    exit_info: "Completed successfully",
  },
  {
    backup_id: "bp-20260708-200000",
    operation: "restore",
    timestamp: "2026-07-08T20:00:00Z",
    source_root: "C:\\Users\\Tony\\Documents",
    dest_path: "D:\\Backups\\Documents",
    job_name: "Documents",
    file_count: 15234,
    total_bytes: 26306674688,
    duration_ms: 240000,
    status: "success",
    exit_info: "Completed successfully",
  },
  {
    backup_id: "bp-20260708-180000",
    operation: "backup",
    timestamp: "2026-07-08T18:00:00Z",
    source_root: "C:\\Configs",
    dest_path: "E:\\Backups\\Configs",
    job_name: "Server Configs",
    file_count: 0,
    total_bytes: 0,
    duration_ms: 500,
    status: "failure",
    exit_info: "I/O error",
  },
  {
    backup_id: "bp-20260708-060000",
    operation: "verify",
    timestamp: "2026-07-08T06:00:00Z",
    source_root: "C:\\Users\\Tony\\Documents",
    dest_path: "D:\\Backups\\Documents",
    job_name: "Documents",
    file_count: 15234,
    total_bytes: 26306674688,
    duration_ms: 90000,
    status: "success",
    exit_info: "Completed successfully",
  },
  {
    backup_id: "bp-20260707-220000",
    operation: "backup",
    timestamp: "2026-07-07T22:00:00Z",
    source_root: "C:\\Users\\Tony\\Documents",
    dest_path: "D:\\Backups\\Documents",
    job_name: "Documents",
    file_count: 15100,
    total_bytes: 26109564928,
    duration_ms: 175000,
    status: "success",
    exit_info: "Completed successfully",
  },
  {
    backup_id: "bp-20260707-210000",
    operation: "backup",
    timestamp: "2026-07-07T21:00:00Z",
    source_root: "C:\\Users\\Tony\\Projects",
    dest_path: "D:\\Backups\\Projects",
    job_name: "Projects",
    file_count: 88100,
    total_bytes: 165789560832,
    duration_ms: 470000,
    status: "partial",
    exit_info: "General failure",
  },
  {
    backup_id: "bp-20260707-060000",
    operation: "backup",
    timestamp: "2026-07-07T06:00:00Z",
    source_root: "C:\\Users\\Tony\\Documents",
    dest_path: "D:\\Backups\\Documents",
    job_name: "Documents",
    file_count: 15000,
    total_bytes: 25950000000,
    duration_ms: 170000,
    status: "success",
    exit_info: "Completed successfully",
  },
];

// const MOCK_TOTAL = MOCK_RECORDS.length;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Query operation history with optional filters.
export async function queryHistory(filter?: HistoryFilter): Promise<{ records: HistoryRecordFormatted[]; total: number }> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    let filtered = MOCK_RECORDS;
    if (filter?.operation) {
      filtered = filtered.filter((r) => r.operation === filter.operation);
    }
    const limit = filter?.limit ?? 50;
    const paged = filtered.slice(0, limit);
    return {
      records: paged.map(formatRecord),
      total: filtered.length,
    };
  }

  try {
    const result = await invoke<HistoryQueryResult>("query_history", { filter: filter ?? {} });
    return {
      records: result.records.map(formatRecord),
      total: result.total,
    };
  } catch (err) {
    console.error("Failed to query history:", err);
    throw err;
  }
}

/// List available operation types for the filter UI.
export async function listHistoryOperationTypes(): Promise<string[]> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    return ["backup", "restore", "verify"];
  }

  try {
    return await invoke<string[]>("list_history_operation_types");
  } catch (err) {
    console.error("Failed to list history operation types:", err);
    return ["backup", "restore", "verify"];
  }
}
