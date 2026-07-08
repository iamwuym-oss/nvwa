// ============================================================================
// backupApi.ts -- Backup page API bridge
//
// This file is the ONLY place where the Backup page talks to the backend.
// It provides:
//   - listJobs() — list all configured backup jobs
//   - runBackup(jobName) — execute a backup for a job
//
// The Backup.tsx page never imports mockData.ts or calls invoke() directly.
// ============================================================================

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// TypeScript interfaces matching Rust src/app/models/backup.rs
// ---------------------------------------------------------------------------

export type BackupJobStatus = "Active" | "NeverRun" | "Error" | "Misconfigured";

export interface BackupJobView {
  name: string;
  source: string;
  dest: string;
  compress: boolean;
  retention: string;
  status: BackupJobStatus;
  last_backup_time: string | null;
  last_backup_status: string | null;
  last_backup_files: number;
  last_backup_bytes: number;
}

export interface BackupResult {
  backup_id: string;
  timestamp: string;
  file_count: number;
  total_bytes: number;
  duration_ms: number;
  status: string;
  error: string | null;
}

// ---------------------------------------------------------------------------
// Formatted view model that the Backup page components consume
// ---------------------------------------------------------------------------

export interface BackupJobViewFormatted {
  name: string;
  source: string;
  dest: string;
  compress: boolean;
  retention: string;
  status: BackupJobStatus;
  statusLabel: string;
  statusColor: "green" | "gray" | "red" | "yellow";
  lastBackupTime: string | null;
  lastBackupStatus: string | null;
  lastBackupFiles: number;
  lastBackupBytes: string;
  canRun: boolean;
}

/// Format bytes into human-readable string
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(1)} ${units[i]}`;
}

/// Map backend job status to UI display values
function statusToDisplay(status: BackupJobStatus): {
  label: string;
  color: "green" | "gray" | "red" | "yellow";
  canRun: boolean;
} {
  switch (status) {
    case "Active":
      return { label: "Active", color: "green", canRun: true };
    case "NeverRun":
      return { label: "Never Run", color: "gray", canRun: true };
    case "Error":
      return { label: "Error", color: "red", canRun: true };
    case "Misconfigured":
      return { label: "Misconfigured", color: "yellow", canRun: false };
  }
}

/// Convert a raw BackupJobView into the formatted version used by UI components
function formatJobView(job: BackupJobView): BackupJobViewFormatted {
  const display = statusToDisplay(job.status);
  return {
    name: job.name,
    source: job.source,
    dest: job.dest,
    compress: job.compress,
    retention: job.retention,
    status: job.status,
    statusLabel: display.label,
    statusColor: display.color,
    lastBackupTime: job.last_backup_time,
    lastBackupStatus: job.last_backup_status,
    lastBackupFiles: job.last_backup_files,
    lastBackupBytes: formatBytes(job.last_backup_bytes),
    canRun: display.canRun,
  };
}

// ---------------------------------------------------------------------------
// Mock data for UI development (only used when VITE_MOCK_DATA=true)
// ---------------------------------------------------------------------------

const MOCK_JOBS: BackupJobView[] = [
  {
    name: "Documents",
    source: "C:\\Users\\Tony\\Documents",
    dest: "D:\\Backups\\Documents",
    compress: true,
    retention: "Keep last 7 versions",
    status: "Active",
    last_backup_time: "2026-07-07T22:00:00Z",
    last_backup_status: "success",
    last_backup_files: 15234,
    last_backup_bytes: 26306674688,
  },
  {
    name: "Projects",
    source: "C:\\Users\\Tony\\Projects",
    dest: "D:\\Backups\\Projects",
    compress: true,
    retention: "Keep last 30 versions",
    status: "Active",
    last_backup_time: "2026-07-07T21:00:00Z",
    last_backup_status: "success",
    last_backup_files: 89234,
    last_backup_bytes: 167675453440,
  },
  {
    name: "Server Configs",
    source: "C:\\Configs",
    dest: "E:\\Backups\\Configs",
    compress: false,
    retention: "Keep 90 days",
    status: "Error",
    last_backup_time: "2026-07-06T08:00:00Z",
    last_backup_status: "failure",
    last_backup_files: 0,
    last_backup_bytes: 0,
  },
  {
    name: "New Project",
    source: "D:\\NewProject",
    dest: "F:\\Backups\\NewProject",
    compress: true,
    retention: "Keep all",
    status: "NeverRun",
    last_backup_time: null,
    last_backup_status: null,
    last_backup_files: 0,
    last_backup_bytes: 0,
  },
];

const MOCK_RESULT: BackupResult = {
  backup_id: "mock-bp-001",
  timestamp: "2026-07-08T09:00:00Z",
  file_count: 15234,
  total_bytes: 26306674688,
  duration_ms: 180000,
  status: "success",
  error: null,
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all configured backup jobs with UI-formatted results.
export async function listJobs(): Promise<BackupJobViewFormatted[]> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    return MOCK_JOBS.map(formatJobView);
  }

  try {
    const jobs = await invoke<BackupJobView[]>("list_backup_jobs");
    return jobs.map(formatJobView);
  } catch (err) {
    console.error("Failed to list backup jobs:", err);
    throw err;
  }
}

/// Execute a backup for the given job name.
/// Returns the BackupResult on success.
export async function runBackup(jobName: string): Promise<BackupResult> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    await new Promise((r) => setTimeout(r, 2000));
    return { ...MOCK_RESULT, backup_id: `mock-bp-${Date.now()}` };
  }

  try {
    return await invoke<BackupResult>("run_backup", { jobName });
  } catch (err) {
    console.error(`Failed to run backup for job '${jobName}':`, err);
    throw err;
  }
}

/// Get details for a single backup job.
export async function getJobDetail(name: string): Promise<BackupJobViewFormatted> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    const job = MOCK_JOBS.find((j) => j.name === name);
    if (!job) throw new Error(`Job '${name}' not found`);
    return formatJobView(job);
  }

  try {
    const job = await invoke<BackupJobView>("get_backup_job_detail", { name });
    return formatJobView(job);
  } catch (err) {
    console.error(`Failed to get job detail for '${name}':`, err);
    throw err;
  }
}
