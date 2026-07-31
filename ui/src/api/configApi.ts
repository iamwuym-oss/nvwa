// ============================================================================
// configApi.ts -- Settings page API bridge for job configuration CRUD
//
// This file is the ONLY place where the Settings page talks to the backend.
// It provides:
//   - listJobConfigs() 鈥?list all job configurations
//   - getJobConfig(name) 鈥?get a single job config
//   - createJobConfig(request) 鈥?create a new job config
//   - updateJobConfig(name, request) 鈥?update an existing job config
//   - deleteJobConfig(name) 鈥?delete a job config
//
// The Settings.tsx page never calls invoke() directly.
// ============================================================================

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// TypeScript interfaces matching Rust src/app/models/config_job.rs
// ---------------------------------------------------------------------------

export interface JobConfigView {
  name: string;
  source: string;
  dest: string;
  compress: boolean;
  retention_keep_count: number | null;
  retention_keep_days: number | null;
  schedule_id: string | null;
  storage_type: string | null;
  repository_id: string | null;
}

export interface JobConfigRequest {
  name: string;
  source: string;
  dest: string;
  compress: boolean;
  retention_keep_count: number | null;
  retention_keep_days: number | null;
  schedule_id: string | null;
  storage_type: string | null;
  repository_id: string | null;
}

// ---------------------------------------------------------------------------
// Mock data for UI development
// ---------------------------------------------------------------------------

const MOCK_CONFIGS: JobConfigView[] = [
  {
    name: "Documents",
    source: "C:\\Users\\Tony\\Documents",
    dest: "D:\\Backups\\Documents",
    compress: true,
    retention_keep_count: 7,
    retention_keep_days: 30,
    schedule_id: "daily-evening",
    storage_type: "repository",
    repository_id: null,
  },
  {
    name: "Projects",
    source: "C:\\Users\\Tony\\Projects",
    dest: "D:\\Backups\\Projects",
    compress: true,
    retention_keep_count: 10,
    retention_keep_days: null,
    schedule_id: null,
    storage_type: "repository",
    repository_id: null,
  },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function useMock(): boolean {
  return import.meta.env.VITE_MOCK_DATA === "true" ||
         import.meta.env.VITE_MOCK_DATA === true;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all job configurations, sorted by name.
export async function listJobConfigs(): Promise<JobConfigView[]> {
  if (useMock()) return [...MOCK_CONFIGS];
  try {
    return await invoke<JobConfigView[]>("list_job_configs");
  } catch (err) {
    console.error("Failed to list job configs:", err);
    throw err;
  }
}

/// Get a single job configuration by name.
export async function getJobConfig(name: string): Promise<JobConfigView> {
  if (useMock()) {
    const cfg = MOCK_CONFIGS.find((c) => c.name === name);
    if (!cfg) throw new Error(`Job '${name}' not found`);
    return cfg;
  }
  try {
    return await invoke<JobConfigView>("get_job_config", { name });
  } catch (err) {
    console.error(`Failed to get job config '${name}':`, err);
    throw err;
  }
}

/// Create a new job configuration.
export async function createJobConfig(request: JobConfigRequest): Promise<JobConfigView> {
  if (useMock()) {
    await new Promise((r) => setTimeout(r, 300));
    if (MOCK_CONFIGS.some((c) => c.name === request.name)) {
      throw new Error(`Job '${request.name}' already exists`);
    }
    const view: JobConfigView = { ...request };
    MOCK_CONFIGS.push(view);
    return view;
  }
  try {
    return await invoke<JobConfigView>("create_job_config", { request });
  } catch (err) {
    console.error("Failed to create job config:", err);
    throw err;
  }
}

/// Update an existing job configuration.
export async function updateJobConfig(name: string, request: JobConfigRequest): Promise<JobConfigView> {
  if (useMock()) {
    await new Promise((r) => setTimeout(r, 300));
    const idx = MOCK_CONFIGS.findIndex((c) => c.name === name);
    if (idx === -1) throw new Error(`Job '${name}' not found`);
    const view: JobConfigView = { ...request };
    MOCK_CONFIGS[idx] = view;
    return view;
  }
  try {
    return await invoke<JobConfigView>("update_job_config", { name, request });
  } catch (err) {
    console.error(`Failed to update job config '${name}':`, err);
    throw err;
  }
}

/// Delete a job configuration by name.
export async function deleteJobConfig(name: string): Promise<void> {
  if (useMock()) {
    await new Promise((r) => setTimeout(r, 300));
    const idx = MOCK_CONFIGS.findIndex((c) => c.name === name);
    if (idx === -1) throw new Error(`Job '${name}' not found`);
    MOCK_CONFIGS.splice(idx, 1);
    return;
  }
  try {
    await invoke("delete_job_config", { name });
  } catch (err) {
    console.error(`Failed to delete job config '${name}':`, err);
    throw err;
  }
}

