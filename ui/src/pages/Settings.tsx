// ============================================================================
// Settings.tsx -- Settings page (future: global application settings)
//
// Backup plan configuration has been moved to the Backup page
// (three-column layout: list | right-side config panel).
//
// This page is reserved for:
//   - Global application configuration
//   - Environment configuration
//   - Theme / language / other preferences
// ============================================================================

import { theme } from "../theme";
import React, { useEffect, useState, useCallback } from "react";
import Badge from "../components/common/Badge";
import ErrorState from "../components/feedback/ErrorState";
import {
  listRepos,
  createRepo,
  verifyRepo,
  RepoInfoResponse,
  VerifyResponse,
} from "../api/repoApi";



// ---------------------------------------------------------------------------
// Inline SVG icons
// ---------------------------------------------------------------------------
function PlusIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>); }
function RefreshIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="23 4 23 10 17 10" /><path d="M20.49 15a9 9 0 11-2.12-9.36L23 10" /></svg>); }
function ShieldIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" /></svg>); }
function AlertIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" /></svg>); }
function CheckIcon() { return (<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12" /></svg>); }
﻿
// ---------------------------------------------------------------------------
// CreateRepoForm inline component
// ---------------------------------------------------------------------------

interface CreateRepoFormProps {
  onCreated: () => void;
  onCancel: () => void;
}

function CreateRepoForm({ onCreated, onCancel }: CreateRepoFormProps) {
  const [name, setName] = useState("");
  const [path, setPath] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async () => {
    setError(null);
    if (!name.trim()) { setError("Repository name is required"); return; }
    if (!path.trim()) { setError("Storage path is required"); return; }
    setSaving(true);
    try {
      await createRepo(name.trim(), path.trim());
      onCreated();
    } catch (err: any) {
      setError(err?.toString() || "Failed to create repository");
    } finally {
      setSaving(false);
    }
  };

  const inputStyle: React.CSSProperties = {
    width: "100%", padding: "10px 12px",
    background: theme.colors.background,
    border: "1px solid " + theme.colors.panelBorder,
    borderRadius: theme.radius.md,
    color: theme.colors.textPrimary,
    fontSize: theme.font.sizeMd,
    fontFamily: theme.font.family,
    outline: "none", boxSizing: "border-box",
  };
  const labelStyle: React.CSSProperties = {
    display: "block", fontSize: theme.font.sizeSm,
    color: theme.colors.textSecondary, marginBottom: "6px", fontWeight: 500,
  };

  return (
    <div style={{
      background: theme.colors.panel,
      border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.xl,
      padding: theme.spacing.xl,
      marginTop: "12px",
    }}>
      <h4 style={{ fontSize: theme.font.sizeMd, fontWeight: 700, color: theme.colors.textPrimary, margin: "0 0 16px 0" }}>
        Create New Repository
      </h4>
      {error && (
        <div style={{
          padding: "10px 14px", background: theme.colors.errorDim,
          border: "1px solid rgba(239,68,68,0.2)", borderRadius: theme.radius.md,
          color: theme.colors.error, fontSize: theme.font.sizeSm,
          marginBottom: "16px", display: "flex", alignItems: "center", gap: "8px",
        }}>
          <AlertIcon /><span>{error}</span>
        </div>
      )}
      <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
        <div>
          <label style={labelStyle}>Repository Name</label>
          <input style={inputStyle} value={name}
            onChange={(e: any) => setName(e.target.value)}
            placeholder="e.g., Main Backup Storage" disabled={saving} />
        </div>
        <div>
          <label style={labelStyle}>Storage Path</label>
          <input style={inputStyle} value={path}
            onChange={(e: any) => setPath(e.target.value)}
            placeholder="D:\\NuwaRepo\\main" disabled={saving} />
        </div>
        <div style={{ display: "flex", gap: "10px", marginTop: "4px" }}>
          <button onClick={handleCreate} disabled={saving}
            style={{ background: theme.colors.primary, border: "none", borderRadius: theme.radius.md, color: "#fff", padding: "10px 20px", cursor: "pointer", fontSize: theme.font.sizeMd, fontFamily: theme.font.family, fontWeight: 600 }}>
            {saving ? "Creating..." : "Create Repository"}
          </button>
          <button onClick={onCancel} disabled={saving}
            style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.md, color: theme.colors.textSecondary, padding: "10px 20px", cursor: "pointer", fontSize: theme.font.sizeMd, fontFamily: theme.font.family }}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// VerifyResultDisplay
// ---------------------------------------------------------------------------

function VerifyResultDisplay({ result }: { result: VerifyResponse | null }) {
  if (!result) return null;
  const color = result.passed ? theme.colors.success : theme.colors.error;
  return (
    <div style={{
      background: result.passed ? theme.colors.successDim : theme.colors.errorDim,
      border: "1px solid " + color + "40",
      borderRadius: theme.radius.md, padding: "12px 16px", marginTop: "12px",
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px" }}>
        {result.passed ? <CheckIcon /> : <AlertIcon />}
        <span style={{ color, fontWeight: 600, fontSize: theme.font.sizeMd }}>
          {result.passed ? "Verification Passed" : "Verification Failed"}
        </span>
      </div>
      <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, margin: 0 }}>
        {result.summary}
      </p>
      {result.details.length > 0 && (
        <ul style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, margin: "8px 0 0 0", paddingLeft: "20px" }}>
          {result.details.map((d, i) => <li key={i}>{d}</li>)}
        </ul>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// RepoCard
// ---------------------------------------------------------------------------

function RepoCard({ repo }: { repo: RepoInfoResponse }) {
  const [verifyResult, setVerifyResult] = useState<VerifyResponse | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const handleVerify = async () => {
    setVerifying(true); setVerifyResult(null);
    try {
      const result = await verifyRepo(repo.id, true);
      setVerifyResult(result);
    } catch (err: any) {
      setVerifyResult({ passed: false, level: "error", checked_at: "", summary: err?.toString() || "Verification failed", details: [], error_count: 1, warning_count: 0 });
    } finally { setVerifying(false); }
  };

  const statusVariant: "success" | "warning" | "error" | "neutral" =
    repo.status === "active" ? "success" : repo.status === "missing" ? "error" : "neutral";

  return (
    <div style={{
      background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg, padding: theme.spacing.lg,
    }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <div style={{ width: "36px", height: "36px", borderRadius: "8px", background: theme.colors.primaryDim, display: "flex", alignItems: "center", justifyContent: "center" }}>
            <ShieldIcon />
          </div>
          <div>
            <h4 style={{ fontSize: theme.font.sizeMd, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>{repo.name}</h4>
            <p style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted, margin: "2px 0 0 0", fontFamily: "monospace" }}>{repo.path}</p>
          </div>
        </div>
        <Badge variant={statusVariant}>{repo.status}</Badge>
      </div>
      <div style={{ display: "flex", gap: "16px", marginTop: "12px", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
        <span>{formatBytes(repo.total_size_bytes)} stored</span>
        <span>{repo.instance_count} instances</span>
        <span>{repo.total_chunks} chunks</span>
        <span>Block: {repo.block_size}</span>
      </div>
      <div style={{ display: "flex", gap: "8px", marginTop: "12px" }}>
        <button onClick={handleVerify} disabled={verifying}
          style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.md, color: theme.colors.textSecondary, padding: "6px 14px", cursor: "pointer", fontSize: theme.font.sizeSm, fontFamily: theme.font.family }}>
          {verifying ? "Verifying..." : "Verify Integrity"}
        </button>
        <button onClick={() => setExpanded(!expanded)}
          style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.md, color: theme.colors.textSecondary, padding: "6px 14px", cursor: "pointer", fontSize: theme.font.sizeSm, fontFamily: theme.font.family }}>
          {expanded ? "Less Info" : "More Info"}
        </button>
      </div>
      {verifyResult && <VerifyResultDisplay result={verifyResult} />}
      {expanded && (
        <div style={{ marginTop: "12px", padding: "12px", background: theme.colors.background, borderRadius: theme.radius.md, fontSize: theme.font.sizeXs, color: theme.colors.textMuted, fontFamily: "monospace", lineHeight: 1.8 }}>
          <div>ID: {repo.id}</div><div>UUID: {repo.repo_uuid}</div>
          <div>Format: v{repo.format_version}</div><div>Compression: {repo.compression}</div>
          <div>Retention: {repo.retention_count} active / {repo.deleted_count} deleted</div>
          <div>Orphans: {repo.orphan_count}</div>
          <div>Capabilities: {repo.capabilities.join(", ") || "none"}</div>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

// ---------------------------------------------------------------------------
// Main Settings Page
// ---------------------------------------------------------------------------

export default function Settings() {
  const [repos, setRepos] = useState<RepoInfoResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);

  const loadRepos = useCallback(async () => {
    setLoading(true); setError(null);
    try {
      const list = await listRepos();
      setRepos(list);
    } catch (err: any) {
      setError(err?.toString() || "Failed to load repositories");
    } finally { setLoading(false); }
  }, []);

  useEffect(() => { loadRepos(); }, [loadRepos]);

  const panelStyle: React.CSSProperties = {
    background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder,
    borderRadius: theme.radius.xl, padding: theme.spacing.xl,
  };

  return (
    <div style={{ padding: theme.spacing.lg, maxWidth: "800px" }}>
      {/* Page header */}
      <div style={{ marginBottom: "24px" }}>
        <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>Settings</h2>
        <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>
          Global application configuration and storage management
        </p>
      </div>

      {/* Section 1: Global Settings (placeholder) */}
      <div style={{ marginBottom: "32px" }}>
        <h3 style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, margin: "0 0 12px 0" }}>Global Settings</h3>
        <div style={{ ...panelStyle, textAlign: "center" }}>
          <div style={{ width: "48px", height: "48px", borderRadius: "50%", background: theme.colors.primaryDim, display: "flex", alignItems: "center", justifyContent: "center", margin: "0 auto 16px auto" }}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke={theme.colors.primary} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
            </svg>
          </div>
          <h3 style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, margin: "0 0 8px 0" }}>Global Settings</h3>
          <p style={{ fontSize: theme.font.sizeMd, color: theme.colors.textSecondary, margin: 0, lineHeight: 1.6 }}>
            Backup plan configuration has been moved to the <strong>Backup</strong> page.
          </p>
          <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "12px", lineHeight: 1.5 }}>
            This section will be used for global application configuration in a future update.
          </p>
        </div>
      </div>

      {/* Section 2: Repository Management */}
      <div>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
          <div>
            <h3 style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>Repository Management</h3>
            <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, margin: "2px 0 0 0" }}>
              Enterprise-grade block storage for backup data
            </p>
          </div>
          <div style={{ display: "flex", gap: "8px" }}>
            <button onClick={loadRepos} disabled={loading}
              style={{ background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.md, color: theme.colors.textSecondary, padding: "8px 14px", cursor: "pointer", display: "flex", alignItems: "center", gap: "6px", fontSize: theme.font.sizeSm, fontFamily: theme.font.family }}>
              <RefreshIcon /> Refresh
            </button>
            <button onClick={() => setShowCreateForm(true)} disabled={showCreateForm}
              style={{ background: theme.colors.primary, border: "none", borderRadius: theme.radius.md, color: "#fff", padding: "8px 14px", cursor: "pointer", display: "flex", alignItems: "center", gap: "6px", fontSize: theme.font.sizeSm, fontFamily: theme.font.family, fontWeight: 600 }}>
              <PlusIcon /> New Repository
            </button>
          </div>
        </div>

        {showCreateForm && (
          <CreateRepoForm onCreated={() => { setShowCreateForm(false); loadRepos(); }} onCancel={() => setShowCreateForm(false)} />
        )}

        {loading && (
          <div style={{ ...panelStyle, textAlign: "center", padding: "40px" }}>
            <p style={{ color: theme.colors.textMuted, fontSize: theme.font.sizeSm, margin: 0 }}>Loading repositories...</p>
          </div>
        )}

        {error && (
          <div style={{ marginTop: "12px" }}>
            <ErrorState message={error} onRetry={loadRepos} />
          </div>
        )}

        {!loading && !error && repos.length === 0 && !showCreateForm && (
          <div style={{ ...panelStyle, textAlign: "center", padding: "40px" }}>
            <div style={{ width: "48px", height: "48px", borderRadius: "50%", background: theme.colors.primaryDim, display: "flex", alignItems: "center", justifyContent: "center", margin: "0 auto 16px auto" }}>
              <ShieldIcon />
            </div>
            <h4 style={{ fontSize: theme.font.sizeMd, fontWeight: 600, color: theme.colors.textPrimary, margin: "0 0 8px 0" }}>No Repositories Yet</h4>
            <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, margin: "0 0 16px 0", lineHeight: 1.5 }}>
              Create a Repository to enable enterprise-grade block storage for your backups.
            </p>
            <button onClick={() => setShowCreateForm(true)}
              style={{ background: theme.colors.primary, border: "none", borderRadius: theme.radius.md, color: "#fff", padding: "10px 20px", cursor: "pointer", fontSize: theme.font.sizeMd, fontFamily: theme.font.family, fontWeight: 600 }}>
              Create First Repository
            </button>
          </div>
        )}

        {!loading && !error && repos.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            {repos.map((repo) => (<RepoCard key={repo.id} repo={repo} />))}
          </div>
        )}
      </div>

      {/* Retention note */}
      <div style={{ marginTop: "24px", padding: "12px 16px", background: "rgba(245,158,11,0.08)", border: "1px solid rgba(245,158,11,0.2)", borderRadius: theme.radius.md, fontSize: theme.font.sizeXs, color: theme.colors.textMuted, lineHeight: 1.6 }}>
        <strong style={{ color: theme.colors.warning }}>Note:</strong> Deleting restore points from a repository does not free disk space. Physical block cleanup will be available in a future release.
      </div>
    </div>
  );
}