// ============================================================================
// fileBrowserApi.ts -- Tauri invoke wrappers for in-app file browser
// ============================================================================

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types (mirrors Rust FsEntry / RootEntry)
// ---------------------------------------------------------------------------

export interface RootEntry {
  label: string;
  path: string;
}

export interface FsEntry {
  name: string;
  path: string;
  is_directory: boolean;
}

// ---------------------------------------------------------------------------
// API
// ---------------------------------------------------------------------------

/** List available drives/roots on the system */
export async function listRoots(): Promise<RootEntry[]> {
  return await invoke<RootEntry[]>("list_roots");
}

/** List child directories of a given path */
export async function listDirectory(path: string): Promise<FsEntry[]> {
  return await invoke<FsEntry[]>("list_directory", { path });
}
