// ============================================================================
// scheduleApi.ts -- Schedule page API bridge
//
// This file is the ONLY place where the Schedule page talks to the backend.
// ============================================================================

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// TypeScript interfaces matching Rust src/app/models/schedule.rs
// ---------------------------------------------------------------------------

export type TriggerType = "Once" | "Daily" | "Weekly" | "Monthly" | "OnLogon";

export type TaskSyncStatus = "Synced" | "Partial" | "NoJobs" | "Unknown";

export interface ScheduleProfileView {
  id: string;
  name: string;
  description: string | null;
  enabled: boolean;
  trigger_type: TriggerType;
  trigger_summary: string;
  trigger_params: string;
  used_by_count: number;
  used_by_jobs: string[];
  next_run_at: string | null;
  last_run_at: string | null;
  last_run_status: string | null;
  task_sync_status: TaskSyncStatus;
  created_at: string | null;
  updated_at: string | null;
}

export interface ScheduleProfileRequest {
  name: string;
  description: string | null;
  enabled: boolean;
  trigger_type: TriggerType;
  trigger_params: string;
}

export interface ScheduleDeleteResult {
  deleted: boolean;
  affected_jobs: string[];
  message: string;
}

// ---------------------------------------------------------------------------
// Mock data for UI development
// ---------------------------------------------------------------------------

const MOCK_SCHEDULES: ScheduleProfileView[] = [
  {
    id: "daily-evening",
    name: "Daily Evening Backup",
    description: "Run every evening at 9 PM",
    enabled: true,
    trigger_type: "Daily",
    trigger_summary: "Daily at 21:00",
    trigger_params: "21:00",
    used_by_count: 2,
    used_by_jobs: ["Documents", "Pictures"],
    next_run_at: "2026-07-10T21:00:00",
    last_run_at: "2026-07-09T21:00:00",
    last_run_status: "success",
    task_sync_status: "Synced",
    created_at: "2026-07-01T10:00:00",
    updated_at: "2026-07-09T10:00:00",
  },
  {
    id: "weekly-sunday",
    name: "Weekly Sunday Backup",
    description: "Full backup every Sunday",
    enabled: true,
    trigger_type: "Weekly",
    trigger_summary: "Weekly on Sun at 22:00",
    trigger_params: "Sun@22:00",
    used_by_count: 1,
    used_by_jobs: ["Projects"],
    next_run_at: "2026-07-13T22:00:00",
    last_run_at: "2026-07-06T22:00:00",
    last_run_status: "success",
    task_sync_status: "Synced",
    created_at: "2026-07-01T10:00:00",
    updated_at: "2026-07-08T12:00:00",
  },
  {
    id: "on-login-backup",
    name: "On Login Backup",
    description: "Back up desktop files after logging in",
    enabled: false,
    trigger_type: "OnLogon",
    trigger_summary: "On logon (no delay)",
    trigger_params: "0",
    used_by_count: 0,
    used_by_jobs: [],
    next_run_at: null,
    last_run_at: null,
    last_run_status: null,
    task_sync_status: "NoJobs",
    created_at: "2026-07-05T08:00:00",
    updated_at: "2026-07-09T09:00:00",
  },
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export async function listSchedules(): Promise<ScheduleProfileView[]> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;
  if (useMock) return MOCK_SCHEDULES;
  try { return await invoke<ScheduleProfileView[]>("list_schedules"); }
  catch (err) { console.error("Failed to list schedules:", err); throw err; }
}

export async function getSchedule(id: string): Promise<ScheduleProfileView> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;
  if (useMock) {
    const s = MOCK_SCHEDULES.find((s) => s.id === id);
    if (!s) throw new Error("Schedule not found: " + id);
    return s;
  }
  try { return await invoke<ScheduleProfileView>("get_schedule", { id }); }
  catch (err) { console.error("Failed to get schedule:", err); throw err; }
}

export async function createSchedule(request: ScheduleProfileRequest): Promise<ScheduleProfileView> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;
  if (useMock) {
    const view: ScheduleProfileView = {
      id: name_to_id(request.name),
      name: request.name,
      description: request.description,
      enabled: request.enabled,
      trigger_type: request.trigger_type,
      trigger_summary: request.trigger_type + " at " + request.trigger_params,
      trigger_params: request.trigger_params,
      used_by_count: 0,
      used_by_jobs: [],
      next_run_at: null,
      last_run_at: null,
      last_run_status: null,
      task_sync_status: "NoJobs",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    return view;
  }
  try { return await invoke<ScheduleProfileView>("create_schedule", { request }); }
  catch (err) { console.error("Failed to create schedule:", err); throw err; }
}

export async function updateSchedule(id: string, request: ScheduleProfileRequest): Promise<ScheduleProfileView> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;
  if (useMock) return createSchedule(request); // simplified mock
  try { return await invoke<ScheduleProfileView>("update_schedule", { id, request }); }
  catch (err) { console.error("Failed to update schedule:", err); throw err; }
}

export async function deleteSchedule(id: string, confirmed: boolean): Promise<ScheduleDeleteResult> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;
  if (useMock) {
    if (!confirmed) {
      const s = MOCK_SCHEDULES.find((s) => s.id === id);
      const affected = s?.used_by_jobs ?? [];
      if (affected.length > 0) {
        return { deleted: false, affected_jobs: affected, message: "Schedule is used by " + affected.length + " job(s). Confirm deletion." };
      }
    }
    return { deleted: true, affected_jobs: [], message: "Schedule deleted." };
  }
  try { return await invoke<ScheduleDeleteResult>("delete_schedule", { id, confirmed }); }
  catch (err) { console.error("Failed to delete schedule:", err); throw err; }
}

export async function enableSchedule(id: string): Promise<ScheduleProfileView> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;
  if (useMock) {
    const s = MOCK_SCHEDULES.find((s) => s.id === id);
    if (!s) throw new Error("Schedule not found: " + id);
    return { ...s, enabled: true };
  }
  try { return await invoke<ScheduleProfileView>("enable_schedule", { id }); }
  catch (err) { console.error("Failed to enable schedule:", err); throw err; }
}

export async function disableSchedule(id: string): Promise<ScheduleProfileView> {
  const useMock = import.meta.env.VITE_MOCK_DATA === "true" ||
                  import.meta.env.VITE_MOCK_DATA === true;
  if (useMock) {
    const s = MOCK_SCHEDULES.find((s) => s.id === id);
    if (!s) throw new Error("Schedule not found: " + id);
    return { ...s, enabled: false };
  }
  try { return await invoke<ScheduleProfileView>("disable_schedule", { id }); }
  catch (err) { console.error("Failed to disable schedule:", err); throw err; }
}



/// Generate a machine-readable identifier from a human-readable name (mock helper).
function name_to_id(name: string): string {
    return name.trim().toLowerCase().replace(/[^a-z0-9-_]/g, "-").replace(/^-+|-+$/g, "");
}
