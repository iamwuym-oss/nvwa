// ============================================================================
// dashboardApi.ts -- Dashboard API bridge
//
// This file is the ONLY place where the Dashboard page talks to the backend.
// It provides a single getDashboard() function that:
//   - In MOCK mode: returns data from mockData.ts
//   - In PRODUCTION mode: calls Tauri invoke("get_dashboard_overview")
//
// The Dashboard.tsx page never imports mockData.ts or calls invoke() directly.
// ============================================================================

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// TypeScript interfaces matching Rust src/app/models/dashboard.rs
// ---------------------------------------------------------------------------

export type ProtectionStatus = "Protected" | "AtRisk" | "Critical" | "Unknown";
export type HealthStatus = "Healthy" | "Warning" | "Critical";
export type OperationType = "Backup" | "Restore" | "Verify";

export interface BackupSummary {
  timestamp: string;
  status: string;
  job_name: string;
  file_count: number;
  total_bytes: number;
  duration_ms: number;
}

export interface StorageStatus {
  path: string;
  free_bytes: number;
  total_bytes: number;
  used_bytes: number;
  used_pct: number;
}

export interface ActivityRecord {
  timestamp: string;
  operation: OperationType;
  job_name: string | null;
  status: string;
  file_count: number;
  total_bytes: number;
}

export interface DashboardOverview {
  protection_status: ProtectionStatus;
  total_jobs: number;
  last_backup: BackupSummary | null;
  storage: StorageStatus[];
  recent_activity: ActivityRecord[];
  scheduled_count: number;
  health: HealthStatus;
}

// ---------------------------------------------------------------------------
// Formatted view model that the Dashboard components consume
// ---------------------------------------------------------------------------

export interface DashboardView {
  protectionStatus: ProtectionStatus;
  protectionLabel: string;
  protectionSubtitle: string;
  totalJobs: number;
  protectedJobs: number;
  lastBackupAgo: string;
  lastBackupName: string;
  storageUsed: string;
  storageTotal: string;
  storagePercent: number;
  healthStatus: HealthStatus;
  healthDetail: string;
  recentActivity: {
    id: string;
    type: OperationType;
    message: string;
    timestamp: string;
    status: "success" | "warning" | "error";
  }[];
  scheduledCount: number;
  scheduledJobs: { name: string; schedule: string; nextRun: string }[];
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatTimeAgo(iso: string): string {
  try {
    const date = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    if (diffMs < 0) return "just now";
    const diffMin = Math.floor(diffMs / 60000);
    if (diffMin < 1) return "just now";
    if (diffMin < 60) return `${diffMin} min ago`;
    const diffHrs = Math.floor(diffMin / 60);
    if (diffHrs < 24) return `${diffHrs} hr${diffHrs > 1 ? "s" : ""} ago`;
    const diffDays = Math.floor(diffHrs / 24);
    if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? "s" : ""} ago`;
    return date.toLocaleDateString();
  } catch {
    return iso;
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(1)} ${units[i]}`;
}

function operationToMessage(op: OperationType, jobName: string | null, status: string): string {
  const job = jobName || "Unknown";
  switch (op) {
    case "Backup":
      return status === "success"
        ? `${job} completed successfully`
        : `${job} ${status}`;
    case "Restore":
      return `Restore from ${job} ${status}`;
    case "Verify":
      return `Verification of ${job} ${status}`;
  }
}

// ---------------------------------------------------------------------------
// Mock implementation (for UI development without live backend)
// ---------------------------------------------------------------------------

const MOCK_DATA: DashboardOverview = {
  protection_status: "Protected",
  total_jobs: 5,
  last_backup: {
    timestamp: "2026-07-07T10:00:00Z",
    status: "success",
    job_name: "Documents Backup",
    file_count: 15234,
    total_bytes: 26306674688,
    duration_ms: 180000,
  },
  storage: [
    {
      path: "D:\\Backups",
      free_bytes: 1181116006400,
      total_bytes: 2199023255552,
      used_bytes: 1017907249152,
      used_pct: 46.3,
    },
  ],
  recent_activity: [
    { timestamp: "2026-07-07T10:03:00Z", operation: "Backup", job_name: "Documents Backup", status: "success", file_count: 15234, total_bytes: 26306674688 },
    { timestamp: "2026-07-07T09:05:00Z", operation: "Backup", job_name: "Projects Backup", status: "success", file_count: 89234, total_bytes: 167675453440 },
    { timestamp: "2026-07-07T08:01:00Z", operation: "Backup", job_name: "Server Configs", status: "failure", file_count: 0, total_bytes: 0 },
    { timestamp: "2026-07-06T22:00:00Z", operation: "Verify", job_name: "System State", status: "success", file_count: 4523, total_bytes: 8697307136 },
    { timestamp: "2026-07-06T06:00:00Z", operation: "Backup", job_name: "System State", status: "success", file_count: 4523, total_bytes: 8697307136 },
  ],
  scheduled_count: 3,
  health: "Healthy",
};

function mockToView(data: DashboardOverview): DashboardView {
  const ls = data.last_backup;
  const activeJobs = data.recent_activity
    .filter(a => a.operation === "Backup" && a.status === "success")
    .length;

  return {
    protectionStatus: data.protection_status,
    protectionLabel: data.protection_status === "Protected" ? "Your data is protected" :
                     data.protection_status === "AtRisk" ? "Some jobs need attention" :
                     data.protection_status === "Critical" ? "No recent backups" : "Status unknown",
    protectionSubtitle: data.protection_status === "Protected"
      ? "All backup jobs are running normally"
      : data.protection_status === "AtRisk"
        ? "Some backup jobs have issues"
        : "Configure a backup job to protect your data",
    totalJobs: data.total_jobs,
    protectedJobs: activeJobs,
    lastBackupAgo: ls ? formatTimeAgo(ls.timestamp) : "Never",
    lastBackupName: ls ? ls.job_name : "No backups yet",
    storageUsed: data.storage.length > 0 ? formatBytes(data.storage[0].used_bytes) : "N/A",
    storageTotal: data.storage.length > 0 ? formatBytes(data.storage[0].total_bytes) : "N/A",
    storagePercent: data.storage.length > 0 ? data.storage[0].used_pct : 0,
    healthStatus: data.health,
    healthDetail: data.health === "Healthy"
      ? "All systems operational"
      : data.health === "Warning"
        ? "Some issues detected"
        : "Critical issues require attention",
    recentActivity: data.recent_activity.slice(0, 6).map((a, i) => ({
      id: `act-${i}`,
      type: a.operation,
      message: operationToMessage(a.operation, a.job_name, a.status),
      timestamp: formatTimeAgo(a.timestamp),
      status: a.status === "success" ? "success" as const : a.status === "failure" ? "error" as const : "warning" as const,
    })),
    scheduledCount: data.scheduled_count,
    scheduledJobs: [],
  };
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Fetch the DashboardOverview from the backend (or mock).
/// This is the ONLY function Dashboard.tsx calls.
export async function getDashboard(): Promise<DashboardView> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;

  if (useMock) {
    return mockToView(MOCK_DATA);
  }

  try {
    const data = await invoke<DashboardOverview>("get_dashboard_overview");
    return mockToView(data);
  } catch (err) {
    console.error("Failed to fetch dashboard overview, falling back to mock:", err);
    return mockToView(MOCK_DATA);
  }
}

