// Nuwa Backup Desktop GUI — Mock Data
// Provides realistic sample data for UI development without live Rust backend

export interface BackupJob {
  id: string;
  name: string;
  source: string;
  destination: string;
  schedule: string;
  lastRun: string;
  status: "success" | "warning" | "error" | "running";
  totalSize: string;
  protected: boolean;
}

export interface BackupPoint {
  id: string;
  jobName: string;
  timestamp: string;
  size: string;
  files: number;
  status: "success" | "warning" | "error";
}

export interface ActivityEntry {
  id: string;
  type: "backup" | "restore" | "verify" | "prune" | "schedule";
  message: string;
  timestamp: string;
  status: "success" | "warning" | "error" | "info";
}

export interface StorageInfo {
  total: string;
  used: string;
  free: string;
  usagePercent: number;
}

export interface HealthStatus {
  overall: "healthy" | "warning" | "critical";
  lastCheck: string;
  checks: { name: string; status: "ok" | "warn" | "fail"; message: string }[];
}

export interface ScheduledJob {
  id: string;
  name: string;
  schedule: string;
  nextRun: string;
  enabled: boolean;
}

export const mockJobs: BackupJob[] = [
  { id: "job-1", name: "Documents Backup", source: "C:\\Users\\Administrator\\Documents", destination: "D:\\Backups\\Documents", schedule: "Daily at 02:00", lastRun: "2026-07-06 02:00", status: "success", totalSize: "24.5 GB", protected: true },
  { id: "job-2", name: "Projects Backup",    source: "D:\\Projects",                    destination: "E:\\Backups\\Projects",   schedule: "Daily at 03:00", lastRun: "2026-07-06 03:00", status: "success", totalSize: "156.2 GB", protected: true },
  { id: "job-3", name: "System State",       source: "C:\\Windows\\System32\\config", destination: "D:\\Backups\\System",    schedule: "Weekly on Sun",   lastRun: "2026-07-05 04:00", status: "warning", totalSize: "8.1 GB", protected: false },
  { id: "job-4", name: "Photos Archive",     source: "E:\\Photos",                      destination: "F:\\Backups\\Photos",    schedule: "Manual only",     lastRun: "2026-06-28 14:30", status: "success", totalSize: "312.7 GB", protected: true },
  { id: "job-5", name: "Server Configs",     source: "C:\\Configs",                     destination: "D:\\Backups\\Configs",   schedule: "Daily at 04:00", lastRun: "2026-07-06 04:00", status: "error", totalSize: "512 MB", protected: false },
];

export const mockBackupHistory: BackupPoint[] = [
  { id: "bp-1", jobName: "Documents Backup", timestamp: "2026-07-06 02:00:00", size: "24.5 GB", files: 15234, status: "success" },
  { id: "bp-2", jobName: "Documents Backup", timestamp: "2026-07-05 02:00:00", size: "24.3 GB", files: 15198, status: "success" },
  { id: "bp-3", jobName: "Documents Backup", timestamp: "2026-07-04 02:00:00", size: "24.1 GB", files: 15102, status: "success" },
  { id: "bp-4", jobName: "Projects Backup",  timestamp: "2026-07-06 03:00:00", size: "156.2 GB", files: 89234, status: "success" },
  { id: "bp-5", jobName: "Projects Backup",  timestamp: "2026-07-05 03:00:00", size: "155.8 GB", files: 89011, status: "success" },
  { id: "bp-6", jobName: "System State",     timestamp: "2026-07-05 04:00:00", size: "8.1 GB", files: 4523, status: "warning" },
  { id: "bp-7", jobName: "Photos Archive",   timestamp: "2026-06-28 14:30:00", size: "312.7 GB", files: 45678, status: "success" },
];

export const mockActivity: ActivityEntry[] = [
  { id: "act-1", type: "backup",   message: "Documents Backup completed successfully",             timestamp: "2026-07-06 02:03:00", status: "success" },
  { id: "act-2", type: "backup",   message: "Projects Backup completed successfully",              timestamp: "2026-07-06 03:05:00", status: "success" },
  { id: "act-3", type: "backup",   message: "Server Configs backup failed — destination full",      timestamp: "2026-07-06 04:01:00", status: "error" },
  { id: "act-4", type: "verify",   message: "Verified backup point bp-6 — 3 files with warnings",  timestamp: "2026-07-05 05:00:00", status: "warning" },
  { id: "act-5", type: "prune",    message: "Pruned 2 old backup points, freed 48.2 GB",            timestamp: "2026-07-05 06:00:00", status: "success" },
  { id: "act-6", type: "backup",   message: "System State backup completed with warnings",          timestamp: "2026-07-05 04:12:00", status: "warning" },
  { id: "act-7", type: "schedule", message: "Scheduled backup 'Documents Backup' triggered",        timestamp: "2026-07-04 02:00:00", status: "info" },
  { id: "act-8", type: "restore",  message: "Restored Projects Backup to D:\\Projects_Restored",  timestamp: "2026-07-03 15:30:00", status: "success" },
];

export const mockStorage: StorageInfo = {
  total: "2.0 TB",
  used: "892.5 GB",
  free: "1.1 TB",
  usagePercent: 43.6,
};

export const mockHealth: HealthStatus = {
  overall: "healthy",
  lastCheck: "2026-07-07 00:00:00",
  checks: [
    { name: "Disk Space",        status: "ok",  message: "All backup destinations have sufficient space" },
    { name: "Backup Integrity",  status: "ok",  message: "All backup points verified in last 24h" },
    { name: "Scheduled Tasks",   status: "ok",  message: "All scheduled jobs executed on time" },
    { name: "System Resources",  status: "warn",message: "CPU at 72% during last backup window" },
  ],
};

export const mockScheduledJobs: ScheduledJob[] = [
  { id: "sched-1", name: "Documents Backup", schedule: "Daily at 02:00",  nextRun: "2026-07-08 02:00", enabled: true },
  { id: "sched-2", name: "Projects Backup",  schedule: "Daily at 03:00",  nextRun: "2026-07-08 03:00", enabled: true },
  { id: "sched-3", name: "System State",     schedule: "Weekly on Sun at 04:00", nextRun: "2026-07-12 04:00", enabled: false },
  { id: "sched-4", name: "Server Configs",   schedule: "Daily at 04:00",  nextRun: "2026-07-08 04:00", enabled: true },
];
