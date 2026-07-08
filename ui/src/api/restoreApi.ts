// ============================================================================
// restoreApi.ts -- Restore page API bridge
//
// This file is the ONLY place where the Restore page talks to the backend.
// It provides:
//   - listRestorePoints() — list all available backup points
//   - getRestorePreview(backupId) — preview files in a backup point
//   - executeRestore(request) — run a restore operation
//
// The Restore.tsx page never imports mockData.ts or calls invoke() directly.
// ============================================================================

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// TypeScript interfaces matching Rust src/app/models/restore.rs
// ---------------------------------------------------------------------------

export interface RestorePointView {
  backup_id: string;
  job_name: string | null;
  timestamp: string;
  source_root: string;
  dest_path: string;
  file_count: number;
  total_bytes: number;
  status: string;
}

export interface RestoreFileEntry {
  relative_path: string;
  size_bytes: number;
  modified_time: string;
}

export interface RestorePreview {
  point: RestorePointView;
  files: RestoreFileEntry[];
  total_files: number;
  total_bytes: number;
}

export interface RestoreRequest {
  backup_id: string;
  dest: string;
  overwrite: boolean;
}

export interface RestoreOperationResult {
  restore_id: string;
  restored_count: number;
  skipped_count: number;
  checksum_failures: number;
  timestamp: string;
  duration_ms: number;
  status: string;
  error: string | null;
}

// ---------------------------------------------------------------------------
// Formatted view model for UI consumption
// ---------------------------------------------------------------------------

export interface RestorePointViewFormatted {
  backupId: string;
  jobName: string | null;
  timestamp: string;
  timestampLabel: string;
  sourceRoot: string;
  destPath: string;
  fileCount: number;
  totalBytes: string;
  status: string;
  statusColor: "success" | "warning" | "error" | "info" | "neutral";
  statusLabel: string;
}

/// Format bytes into human-readable string
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return val.toFixed(1) + " " + units[i];
}

/// Format an ISO timestamp to a short readable label
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

/// Map restore status to badge variant
function statusToDisplay(status: string): { label: string; color: "success" | "warning" | "error" | "info" | "neutral" } {
  switch (status) {
    case "success": return { label: "Success", color: "success" };
    case "failure": return { label: "Failed", color: "error" };
    case "partial": return { label: "Partial", color: "warning" };
    default:        return { label: status, color: "neutral" };
  }
}

/// Convert raw restore point into formatted UI model
function formatPoint(point: RestorePointView): RestorePointViewFormatted {
  const display = statusToDisplay(point.status);
  return {
    backupId: point.backup_id,
    jobName: point.job_name,
    timestamp: point.timestamp,
    timestampLabel: formatTimestamp(point.timestamp),
    sourceRoot: point.source_root,
    destPath: point.dest_path,
    fileCount: point.file_count,
    totalBytes: formatBytes(point.total_bytes),
    status: point.status,
    statusColor: display.color,
    statusLabel: display.label,
  };
}

// ---------------------------------------------------------------------------
// Mock data for UI development (only used when VITE_MOCK_DATA=true)
// ---------------------------------------------------------------------------

const MOCK_RESTORE_POINTS: RestorePointView[] = [
  // Documents - has 3 backup versions
  {
    backup_id: "bp-20260707-220000",
    job_name: "Documents",
    timestamp: "2026-07-07T22:00:00Z",
    source_root: "C:\\Users\\Tony\\Documents",
    dest_path: "D:\\Backups\\Documents",
    file_count: 15234,
    total_bytes: 26306674688,
    status: "success",
  },
  {
    backup_id: "bp-20260706-220000",
    job_name: "Documents",
    timestamp: "2026-07-06T22:00:00Z",
    source_root: "C:\\Users\\Tony\\Documents",
    dest_path: "D:\\Backups\\Documents",
    file_count: 14890,
    total_bytes: 25894125568,
    status: "success",
  },
  {
    backup_id: "bp-20260705-220000",
    job_name: "Documents",
    timestamp: "2026-07-05T22:00:00Z",
    source_root: "C:\\Users\\Tony\\Documents",
    dest_path: "D:\\Backups\\Documents",
    file_count: 14750,
    total_bytes: 25673856000,
    status: "success",
  },
  // Projects - has 2 backup versions
  {
    backup_id: "bp-20260707-210000",
    job_name: "Projects",
    timestamp: "2026-07-07T21:00:00Z",
    source_root: "C:\\Users\\Tony\\Projects",
    dest_path: "D:\\Backups\\Projects",
    file_count: 89234,
    total_bytes: 167675453440,
    status: "success",
  },
  {
    backup_id: "bp-20260706-210000",
    job_name: "Projects",
    timestamp: "2026-07-06T21:00:00Z",
    source_root: "C:\\Users\\Tony\\Projects",
    dest_path: "D:\\Backups\\Projects",
    file_count: 88720,
    total_bytes: 166994360320,
    status: "success",
  },
  // Server Configs - single backup
  {
    backup_id: "bp-20260706-080000",
    job_name: "Server Configs",
    timestamp: "2026-07-06T08:00:00Z",
    source_root: "C:\\Configs",
    dest_path: "E:\\Backups\\Configs",
    file_count: 0,
    total_bytes: 0,
    status: "success",
  },
];

const MOCK_PREVIEW_FILES: RestoreFileEntry[] = [
  // Documents - top level
  { relative_path: "Documents\\Report.docx", size_bytes: 245760, modified_time: "2026-07-07T21:30:00Z" },
  { relative_path: "Documents\\Budget.xlsx", size_bytes: 102400, modified_time: "2026-07-07T21:25:00Z" },
  { relative_path: "Documents\\Presentation.pptx", size_bytes: 5242880, modified_time: "2026-07-07T21:20:00Z" },
  { relative_path: "Documents\\Notes.txt", size_bytes: 4096, modified_time: "2026-07-07T21:15:00Z" },
  // Projects - includes nested directories
  { relative_path: "Projects\\Source Code\\main.rs", size_bytes: 15360, modified_time: "2026-07-07T18:00:00Z" },
  { relative_path: "Projects\\Source Code\\types.rs", size_bytes: 8192, modified_time: "2026-07-07T17:55:00Z" },
  { relative_path: "Projects\\Source Code\\utils.rs", size_bytes: 12288, modified_time: "2026-07-07T17:50:00Z" },
  { relative_path: "Projects\\Config\\settings.toml", size_bytes: 2048, modified_time: "2026-07-07T16:30:00Z" },
  { relative_path: "Projects\\Config\\database.yml", size_bytes: 1024, modified_time: "2026-07-07T16:25:00Z" },
  { relative_path: "Projects\\README.md", size_bytes: 4096, modified_time: "2026-07-07T16:00:00Z" },
  // Photos - deep nesting
  { relative_path: "Photos\\Vacation\\beach.jpg", size_bytes: 4194304, modified_time: "2026-07-06T12:00:00Z" },
  { relative_path: "Photos\\Vacation\\sunset.png", size_bytes: 8388608, modified_time: "2026-07-06T18:30:00Z" },
  { relative_path: "Photos\\Vacation\\family\\group.jpg", size_bytes: 3145728, modified_time: "2026-07-06T14:00:00Z" },
  { relative_path: "Photos\\Vacation\\family\\selfie.jpg", size_bytes: 2097152, modified_time: "2026-07-06T15:00:00Z" },
  { relative_path: "Photos\\Screenshots\\error.png", size_bytes: 524288, modified_time: "2026-07-05T10:00:00Z" },
  // Root level files
  { relative_path: "config.json", size_bytes: 512, modified_time: "2026-07-01T08:00:00Z" },
  { relative_path: "backup.log", size_bytes: 10240, modified_time: "2026-07-07T22:00:00Z" },
];

const MOCK_RESTORE_RESULT: RestoreOperationResult = {
  restore_id: "rst-20260708090000",
  restored_count: 15234,
  skipped_count: 0,
  checksum_failures: 0,
  timestamp: "2026-07-08T09:00:00Z",
  duration_ms: 120000,
  status: "success",
  error: null,
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all available restore points (newest first).
export async function listRestorePoints(): Promise<RestorePointViewFormatted[]> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    return MOCK_RESTORE_POINTS.map(formatPoint);
  }

  try {
    const points = await invoke<RestorePointView[]>("list_restore_points");
    return points.map(formatPoint);
  } catch (err) {
    console.error("Failed to list restore points:", err);
    throw err;
  }
}

/// Get a preview of files that would be restored from a backup point.
export async function getRestorePreview(backupId: string): Promise<RestorePreview> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    const point = MOCK_RESTORE_POINTS.find((p) => p.backup_id === backupId);
    if (!point) throw new Error("Backup point '" + backupId + "' not found");
    return {
      point,
      files: MOCK_PREVIEW_FILES,
      total_files: MOCK_PREVIEW_FILES.length,
      total_bytes: MOCK_PREVIEW_FILES.reduce((sum, f) => sum + f.size_bytes, 0),
    };
  }

  try {
    return await invoke<RestorePreview>("get_restore_preview", { backupId });
  } catch (err) {
    console.error("Failed to get restore preview for '" + backupId + "':", err);
    throw err;
  }
}

/// Execute a restore operation.
export async function executeRestore(request: RestoreRequest): Promise<RestoreOperationResult> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    await new Promise((r) => setTimeout(r, 2000));
    return {
      ...MOCK_RESTORE_RESULT,
      restore_id: "rst-" + Date.now(),
      timestamp: new Date().toISOString(),
    };
  }

  try {
    return await invoke<RestoreOperationResult>("execute_restore", { request });
  } catch (err) {
    console.error("Failed to execute restore:", err);
    throw err;
  }
}
