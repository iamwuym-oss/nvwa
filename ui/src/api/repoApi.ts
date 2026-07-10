// ============================================================================
// repoApi.ts -- Repository management API bridge
//
// This file is the ONLY place where the UI talks to the backend for
// Repository operations. It provides:
//   - createRepo(name, path) — create and register a Repository
//   - listRepos() — list all registered Repositories
//   - getRepoInfo(id) — get detailed info for one Repository
//   - verifyRepo(id, quick) — run integrity verification
//
// The UI pages never import mockData.ts or call invoke() directly for repos.
// ============================================================================

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// TypeScript interfaces matching Rust src/app/models/repo.rs
// ---------------------------------------------------------------------------

export interface RepoInfoResponse {
  id: string;
  name: string;
  path: string;
  repo_uuid: string;
  format_version: number;
  created_at: string;
  block_size: string;
  compression: string;
  capabilities: string[];
  total_chunks: number;
  total_size_bytes: number;
  instance_count: number;
  retention_count: number;
  deleted_count: number;
  orphan_count: number;
  status: string;
}

export interface VerifyResponse {
  passed: boolean;
  level: string;
  checked_at: string;
  summary: string;
  details: string[];
  error_count: number;
  warning_count: number;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a new Repository at the given path and register it.
export async function createRepo(name: string, path: string): Promise<RepoInfoResponse> {
  return await invoke<RepoInfoResponse>("create_repo", { name, path });
}

/// List all registered Repositories with live status.
export async function listRepos(): Promise<RepoInfoResponse[]> {
  return await invoke<RepoInfoResponse[]>("list_repos");
}

/// Get detailed information for a single Repository.
export async function getRepoInfo(id: string): Promise<RepoInfoResponse> {
  return await invoke<RepoInfoResponse>("get_repo_info", { id });
}

/// Run integrity verification on a Repository.
export async function verifyRepo(id: string, quick: boolean): Promise<VerifyResponse> {
  return await invoke<VerifyResponse>("verify_repo", { id, quick });
}
