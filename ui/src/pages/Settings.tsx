// ============================================================================
// Settings.tsx -- Settings page with Backup Plan configuration
//
// Data flow:
//   useEffect -> listJobConfigs() -> JobConfigView[] -> render plan list
//   "Create Backup Plan" -> form modal -> createJobConfig() -> refresh
//   "Edit" -> form modal (prefilled) -> updateJobConfig() -> refresh
//   "Delete" -> confirm -> deleteJobConfig() -> refresh
//
// States handled:
//   - Loading: skeleton cards
//   - Error:   error card with retry button
//   - Empty:   guidance screen to create first backup plan
//   - Data:    plan list with edit/delete actions
//   - Form:    modal for create/edit
// ============================================================================

import { useEffect, useState, useCallback } from "react";
import Button from "../components/common/Button";
import Badge from "../components/common/Badge";
import EmptyState from "../components/feedback/EmptyState";
import ErrorState from "../components/feedback/ErrorState";
import { theme } from "../theme";
import {
  listJobConfigs,
  createJobConfig,
  updateJobConfig,
  deleteJobConfig,
  JobConfigView,
  JobConfigRequest,
} from "../api/configApi";

// ---------------------------------------------------------------------------
// Inline SVG icons
// ---------------------------------------------------------------------------

function ShieldIcon() {
  return (
    <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
    </svg>
  );
}

function PlusIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
    </svg>
  );
}

function EditIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
      <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="3 6 5 6 21 6"/>
      <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
    </svg>
  );
}

function RefreshIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="23 4 23 10 17 10"/>
      <path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/>
    </svg>
  );
}

function FolderIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5c0-1.1.9-2 2-2h5l2 3h9a2 2 0 012 2v11z"/>
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="20 6 9 17 4 12"/>
    </svg>
  );
}

function AlertIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10"/>
      <line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
    </svg>
  );
}

// ---------------------------------------------------------------------------
// Skeleton card for loading state
// ---------------------------------------------------------------------------

const sk: React.CSSProperties = {
  background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)",
  backgroundSize: "200% 100%",
  animation: "skeletonPulse 1.5s infinite",
};

function SkeletonPlanCard() {
  return (
    <div style={{
      background: theme.colors.panel,
      border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg,
      padding: theme.spacing.lg,
    }}>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "16px" }}>
        <div style={{ width: "160px", ...sk }} />
        <div style={{ width: "70px", height: "22px", borderRadius: theme.radius.full, ...sk }} />
      </div>
      <div style={{ display: "flex", gap: "24px", marginBottom: "12px" }}>
        <div style={{ width: "200px", ...sk }} />
        <div style={{ width: "200px", ...sk }} />
      </div>
      <div style={{ width: "120px", ...sk }} />
    </div>
  );
}


// ---------------------------------------------------------------------------
// Backup Plan Form (create / edit)
// ---------------------------------------------------------------------------

interface PlanFormProps {
  initial?: JobConfigView;
  onSave: (request: JobConfigRequest) => Promise<void>;
  onCancel: () => void;
  saving: boolean;
  error: string | null;
}

function PlanForm({ initial, onSave, onCancel, saving, error }: PlanFormProps) {
  const [name, setName] = useState(initial?.name ?? "");
  const [source, setSource] = useState(initial?.source ?? "");
  const [dest, setDest] = useState(initial?.dest ?? "");
  const [compress, setCompress] = useState(initial?.compress ?? true);
  const [keepCount, setKeepCount] = useState(initial?.retention_keep_count?.toString() ?? "");
  const [keepDays, setKeepDays] = useState(initial?.retention_keep_days?.toString() ?? "");
  const [formError, setFormError] = useState<string | null>(null);

  const handleSubmit = async () => {
    setFormError(null);

    if (!name.trim()) { setFormError("Plan name is required"); return; }

    const request: JobConfigRequest = {
      name: name.trim(),
      source: source.trim(),
      dest: dest.trim(),
      compress,
      retention_keep_count: keepCount ? parseInt(keepCount) || null : null,
      retention_keep_days: keepDays ? parseInt(keepDays) || null : null,
    };

    await onSave(request);
  };

  const inputStyle: React.CSSProperties = {
    width: "100%",
    padding: "10px 12px",
    background: theme.colors.background,
    border: "1px solid " + theme.colors.panelBorder,
    borderRadius: theme.radius.md,
    color: theme.colors.textPrimary,
    fontSize: theme.font.sizeMd,
    fontFamily: theme.font.family,
    outline: "none",
    boxSizing: "border-box",
  };

  const labelStyle: React.CSSProperties = {
    display: "block",
    fontSize: theme.font.sizeSm,
    color: theme.colors.textSecondary,
    marginBottom: "6px",
    fontWeight: 500,
  };

  return (
    <div style={{ maxWidth: "600px" }}>
      <div style={{ marginBottom: "20px" }}>
        <h3 style={{ fontSize: theme.font.sizeXl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>
          {initial ? "Edit Backup Plan" : "Create Backup Plan"}
        </h3>
        <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "4px", marginBottom: 0 }}>
          {initial ? "Update the backup plan configuration." : "Configure a new backup job to protect your data."}
        </p>
      </div>

      {(formError || error) && (
        <div style={{
          padding: "10px 14px",
          background: theme.colors.errorDim,
          border: "1px solid rgba(239,68,68,0.2)",
          borderRadius: theme.radius.md,
          color: theme.colors.error,
          fontSize: theme.font.sizeSm,
          marginBottom: "16px",
          display: "flex",
          alignItems: "center",
          gap: "8px",
        }}>
          <AlertIcon />
          <span>{formError || error}</span>
        </div>
      )}

      <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
        {/* Plan Name */}
        <div>
          <label style={labelStyle}>Plan Name</label>
          <input
            style={inputStyle}
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., Documents Backup"
            disabled={saving}
          />
        </div>

        {/* Source Path */}
        <div>
          <label style={labelStyle}>Source Path</label>
          <input
            style={inputStyle}
            value={source}
            onChange={(e) => setSource(e.target.value)}
            placeholder="C:\Users\YourName\Documents"
            disabled={saving}
          />
        </div>

        {/* Destination Path */}
        <div>
          <label style={labelStyle}>Destination Path</label>
          <input
            style={inputStyle}
            value={dest}
            onChange={(e) => setDest(e.target.value)}
            placeholder="D:\Backups"
            disabled={saving}
          />
        </div>

        {/* Compression */}
        <div>
          <label style={{ ...labelStyle, marginBottom: "8px" }}>Compression</label>
          <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer", color: theme.colors.textPrimary, fontSize: theme.font.sizeMd }}>
            <input
              type="checkbox"
              checked={compress}
              onChange={(e) => setCompress(e.target.checked)}
              disabled={saving}
              style={{ accentColor: theme.colors.primary }}
            />
            Enable compression (zstd)
          </label>
        </div>

        {/* Retention */}
        <div style={{ display: "flex", gap: "16px" }}>
          <div style={{ flex: 1 }}>
            <label style={labelStyle}>Keep Count</label>
            <input
              style={inputStyle}
              type="number"
              min={0}
              value={keepCount}
              onChange={(e) => setKeepCount(e.target.value)}
              placeholder="e.g., 7"
              disabled={saving}
            />
          </div>
          <div style={{ flex: 1 }}>
            <label style={labelStyle}>Keep Days</label>
            <input
              style={inputStyle}
              type="number"
              min={0}
              value={keepDays}
              onChange={(e) => setKeepDays(e.target.value)}
              placeholder="e.g., 30"
              disabled={saving}
            />
          </div>
        </div>
      </div>

      <div style={{ display: "flex", gap: "12px", marginTop: "24px" }}>
        <Button variant="primary" onClick={handleSubmit} disabled={saving} icon={<CheckIcon />}>
          {saving ? "Saving..." : initial ? "Save Changes" : "Create Plan"}
        </Button>
        <Button variant="secondary" onClick={onCancel} disabled={saving}>
          Cancel
        </Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Plan card component
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Retention display helper
// ---------------------------------------------------------------------------

function retentionLabel(count: number | null, days: number | null): string {
  if (count !== null && days !== null) return "Keep " + count + " versions or " + days + " days";
  if (count !== null) return "Keep " + count + " versions";
  if (days !== null) return "Keep " + days + " days";
  return 'Keep all';
}

interface PlanCardProps {
  plan: JobConfigView;
  onEdit: () => void;
  onDelete: () => void;
}

function PlanCard({ plan, onEdit, onDelete }: PlanCardProps) {
  return (
    <div style={{
      background: theme.colors.panel,
      border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg,
      padding: theme.spacing.lg,
      transition: theme.transition.fast,
    }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.borderColor = theme.colors.primary; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.borderColor = theme.colors.panelBorder; }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "12px" }}>
        <div>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <h3 style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>
              {plan.name}
            </h3>
            <Badge variant={plan.compress ? "info" : "neutral"}>
              {plan.compress ? "Compressed" : "No Compression"}
            </Badge>
          </div>
        </div>
        <div style={{ display: "flex", gap: "6px" }}>
          <button
            onClick={onEdit}
            style={{
              background: "rgba(255,255,255,0.06)",
              border: "1px solid rgba(255,255,255,0.1)",
              borderRadius: theme.radius.md,
              color: theme.colors.textSecondary,
              padding: "6px 10px",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              gap: "4px",
              fontSize: theme.font.sizeSm,
              fontFamily: theme.font.family,
              transition: theme.transition.fast,
            }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.background = theme.colors.panelHover; (e.currentTarget as HTMLElement).style.color = theme.colors.textPrimary; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.06)"; (e.currentTarget as HTMLElement).style.color = theme.colors.textSecondary; }}
          >
            <EditIcon /> Edit
          </button>
          <button
            onClick={onDelete}
            style={{
              background: "transparent",
              border: "1px solid rgba(239,68,68,0.2)",
              borderRadius: theme.radius.md,
              color: theme.colors.error,
              padding: "6px 10px",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              gap: "4px",
              fontSize: theme.font.sizeSm,
              fontFamily: theme.font.family,
              transition: theme.transition.fast,
            }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.background = theme.colors.errorDim; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.background = "transparent"; }}
          >
            <TrashIcon /> Delete
          </button>
        </div>
      </div>

      <div style={{ display: "flex", gap: "24px", flexWrap: "wrap", marginBottom: "8px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
          <FolderIcon />
          <span>From: <span style={{ color: theme.colors.textPrimary, fontFamily: theme.font.mono, fontSize: "12px" }}>{plan.source}</span></span>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
          <SettingsIcon />
          <span>To: <span style={{ color: theme.colors.textPrimary, fontFamily: theme.font.mono, fontSize: "12px" }}>{plan.dest}</span></span>
        </div>
      </div>

      <div style={{ display: "flex", gap: "24px", flexWrap: "wrap" }}>
        <span style={{ fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
          Retention: <span style={{ color: theme.colors.textSecondary }}>{retentionLabel(plan.retention_keep_count, plan.retention_keep_days)}</span>
        </span>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Confirm Delete Dialog
// ---------------------------------------------------------------------------

function ConfirmDeleteDialog({ name, onConfirm, onCancel, deleting }: {
  name: string;
  onConfirm: () => void;
  onCancel: () => void;
  deleting: boolean;
}) {
  return (
    <div style={{
      position: "fixed",
      inset: 0,
      background: theme.colors.overlay,
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      zIndex: 1000,
    }}>
      <div style={{
        background: theme.colors.panel,
        border: "1px solid " + theme.colors.panelBorder,
        borderRadius: theme.radius.xl,
        padding: theme.spacing.xl,
        maxWidth: "420px",
        width: "90%",
      }}>
        <h3 style={{ fontSize: theme.font.sizeXl, fontWeight: 700, color: theme.colors.textPrimary, margin: "0 0 8px 0" }}>
          Delete Backup Plan
        </h3>
        <p style={{ fontSize: theme.font.sizeMd, color: theme.colors.textSecondary, margin: "0 0 24px 0", lineHeight: 1.6 }}>
          Are you sure you want to delete <strong style={{ color: theme.colors.textPrimary }}>{name}</strong>?
          <br />
          Backup history and stored files will not be deleted, but the plan will be removed.
        </p>
        <div style={{ display: "flex", gap: "12px", justifyContent: "flex-end" }}>
          <Button variant="secondary" onClick={onCancel} disabled={deleting}>Cancel</Button>
          <Button variant="danger" onClick={onConfirm} disabled={deleting}>
            {deleting ? "Deleting..." : "Delete Plan"}
          </Button>
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Success Toast
// ---------------------------------------------------------------------------

function Toast({ message, onClose }: { message: string; onClose: () => void }) {
  useEffect(() => {
    const timer = setTimeout(onClose, 4000);
    return () => clearTimeout(timer);
  }, [onClose]);

  return (
    <div style={{
      position: "fixed",
      bottom: "24px",
      right: "24px",
      background: theme.colors.panelElevated,
      border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg,
      padding: "14px 20px",
      display: "flex",
      alignItems: "center",
      gap: "10px",
      color: theme.colors.success,
      fontSize: theme.font.sizeMd,
      fontWeight: 500,
      boxShadow: theme.shadow.elevated,
      zIndex: 1001,
      animation: "slideUp 0.3s ease",
    }}>
      <CheckIcon />
      <span>{message}</span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main Settings Page
// ---------------------------------------------------------------------------

export default function Settings() {
  const [plans, setPlans] = useState<JobConfigView[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingPlan, setEditingPlan] = useState<JobConfigView | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [showConfirmDelete, setShowConfirmDelete] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const fetchPlans = useCallback(() => {
    setLoading(true);
    setError(null);

    listJobConfigs()
      .then((configs) => {
        setPlans(configs);
        setLoading(false);
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : "Failed to load backup plans";
        setError(msg);
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    fetchPlans();
  }, [fetchPlans]);

  const openCreateForm = () => {
    setEditingPlan(null);
    setFormError(null);
    setShowForm(true);
  };

  const openEditForm = (plan: JobConfigView) => {
    setEditingPlan(plan);
    setFormError(null);
    setShowForm(true);
  };

  const closeForm = () => {
    setShowForm(false);
    setEditingPlan(null);
    setFormError(null);
  };

  const handleSave = async (request: JobConfigRequest) => {
    setSaving(true);
    setFormError(null);

    try {
      if (editingPlan) {
        await updateJobConfig(editingPlan.name, request);
        setToastMessage(`Plan '${request.name}' updated successfully`);
      } else {
        await createJobConfig(request);
        setToastMessage(`Plan '${request.name}' created successfully`);
      }
      closeForm();
      await fetchPlans();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Operation failed";
      setFormError(msg);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!showConfirmDelete) return;
    setDeleting(true);

    try {
      await deleteJobConfig(showConfirmDelete);
      setToastMessage(`Plan '${showConfirmDelete}' deleted`);
      setShowConfirmDelete(null);
      await fetchPlans();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Delete failed";
      setFormError(msg);
      setShowConfirmDelete(null);
    } finally {
      setDeleting(false);
    }
  };

  // Inject skeleton animation
  useEffect(() => {
    if (!document.getElementById("settings-skeleton-style")) {
      const style = document.createElement("style");
      style.id = "settings-skeleton-style";
      style.textContent = `
        @keyframes skeletonPulse {
          0% { background-position: 200% 0; }
          100% { background-position: -200% 0; }
        }
        @keyframes slideUp {
          from { transform: translateY(20px); opacity: 0; }
          to { transform: translateY(0); opacity: 1; }
        }
      `;
      document.head.appendChild(style);
    }
  }, []);

  // ---- Loading ----
  if (loading) {
    return (
      <div style={{ padding: theme.spacing.lg, maxWidth: "900px" }}>
        <div style={{ marginBottom: "24px" }}>
          <div style={{ width: "200px", height: "28px", ...sk }} />
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <SkeletonPlanCard /><SkeletonPlanCard />
        </div>
      </div>
    );
  }

  // ---- Error ----
  if (error) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <ErrorState title="Failed to load backup plans" message={error} onRetry={fetchPlans} />
      </div>
    );
  }

  // ---- Empty ----
  if (plans.length === 0 && !showForm) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <EmptyState
          icon={<ShieldIcon />}
          title="No Backup Plans Configured"
          description="Create your first backup plan to start protecting your important data. You can configure source and destination paths, compression, and retention settings."
          primaryAction={{
            label: "Create Backup Plan",
            onClick: openCreateForm,
          }}
        />
        {showForm && (
          <div style={{
            marginTop: "32px",
            background: theme.colors.panel,
            border: "1px solid " + theme.colors.panelBorder,
            borderRadius: theme.radius.xl,
            padding: theme.spacing.xl,
          }}>
            <PlanForm
              onSave={handleSave}
              onCancel={closeForm}
              saving={saving}
              error={formError}
            />
          </div>
        )}
      </div>
    );
  }

  // ---- Data + Form ----
  return (
    <div style={{ padding: theme.spacing.lg, maxWidth: "900px" }}>
      <style>{`
        @keyframes skeletonPulse {
          0% { background-position: 200% 0; }
          100% { background-position: -200% 0; }
        }
        @keyframes slideUp {
          from { transform: translateY(20px); opacity: 0; }
          to { transform: translateY(0); opacity: 1; }
        }
      `}</style>

      {/* Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "24px" }}>
        <div>
          <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>
            Settings
          </h2>
          <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px", marginBottom: 0 }}>
            Backup Plans &middot; {plans.length} plan{plans.length !== 1 ? "s" : ""} configured
          </p>
        </div>
        <div style={{ display: "flex", gap: "8px" }}>
          <Button variant="secondary" size="sm" onClick={fetchPlans} icon={<RefreshIcon />}>Refresh</Button>
          <Button variant="primary" size="sm" onClick={openCreateForm} icon={<PlusIcon />}>Create Plan</Button>
        </div>
      </div>

      {/* Form panel (create/edit) */}
      {showForm && (
        <div style={{
          background: theme.colors.panel,
          border: "1px solid " + theme.colors.primary,
          borderRadius: theme.radius.xl,
          padding: theme.spacing.xl,
          marginBottom: "24px",
        }}>
          <PlanForm
            initial={editingPlan ?? undefined}
            onSave={handleSave}
            onCancel={closeForm}
            saving={saving}
            error={formError}
          />
        </div>
      )}

      {/* Plan list */}
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {plans.map((plan) => (
          <PlanCard
            key={plan.name}
            plan={plan}
            onEdit={() => openEditForm(plan)}
            onDelete={() => setShowConfirmDelete(plan.name)}
          />
        ))}
      </div>

      {/* Delete confirmation */}
      {showConfirmDelete && (
        <ConfirmDeleteDialog
          name={showConfirmDelete}
          onConfirm={handleDelete}
          onCancel={() => setShowConfirmDelete(null)}
          deleting={deleting}
        />
      )}

      {/* Toast */}
      {toastMessage && (
        <Toast message={toastMessage} onClose={() => setToastMessage(null)} />
      )}
    </div>
  );
}
