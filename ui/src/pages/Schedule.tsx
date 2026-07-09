// ============================================================================
// Schedule.tsx -- Schedule Profile management page
//
// Data flow:
//   useEffect -> listSchedules() -> ScheduleProfileView[] -> render cards
//   "Create Schedule" -> modal -> createSchedule() -> refresh
//   "Edit" -> modal (prefilled) -> updateSchedule() -> refresh
//   "Delete" -> check -> confirm -> deleteSchedule() -> refresh
//   "Enable/Disable" -> toggle -> enableSchedule/disableSchedule() -> refresh
//
// States handled:
//   - Loading: skeleton cards with pulse animation
//   - Error:   error card with retry
//   - Empty:   guidance screen to create first schedule
//   - Data:    schedule card grid with actions
//   - Modal:   create/edit form with trigger type selection
//   - Confirm: delete confirmation
// ============================================================================

import { useEffect, useState, useCallback } from "react";
import Button from "../components/common/Button";
import Badge from "../components/common/Badge";
import EmptyState from "../components/feedback/EmptyState";
import ErrorState from "../components/feedback/ErrorState";
import { theme } from "../theme";
import {
  listSchedules,
  createSchedule,
  updateSchedule,
  deleteSchedule,
  enableSchedule,
  disableSchedule,
  ScheduleProfileView,
  ScheduleProfileRequest,
  TriggerType,
} from "../api/scheduleApi";

// ============================================================================
// Inline SVG Icons
// ============================================================================

function PlusIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
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

function PowerIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M18.36 6.64a9 9 0 11-12.73 0"/><line x1="12" y1="2" x2="12" y2="12"/>
    </svg>
  );
}

// ============================================================================
// Trigger type display helpers
// ============================================================================

function triggerBadgeVariant(t: TriggerType): "success" | "warning" | "error" | "info" | "neutral" {
  switch (t) {
    case "Daily":   return "info";
    case "Weekly":  return "success";
    case "Monthly": return "warning";
    case "Once":    return "neutral";
    case "OnLogon": return "neutral";
    default:         return "neutral";
  }
}

const TRIGGER_LABELS: Record<TriggerType, string> = {
  Once: "Once",
  Daily: "Daily",
  Weekly: "Weekly",
  Monthly: "Monthly",
  OnLogon: "On Logon",
};

// ============================================================================
// Skeleton card for loading state
// ============================================================================

function SkeletonCard() {
  const sk = {
    height: "14px",
    borderRadius: theme.radius.sm,
    background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)",
    backgroundSize: "200% 100%",
    animation: "skeletonPulse 1.5s infinite",
  };
  return (
    <div style={{
      background: theme.colors.panel,
      border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg,
      padding: theme.spacing.lg,
    }}>
      <div style={{ width: "60%", ...sk, marginBottom: "12px" }} />
      <div style={{ width: "40%", ...sk, marginBottom: "8px" }} />
      <div style={{ width: "80%", ...sk }} />
    </div>
  );
}

// ============================================================================
// Schedule card component
// ============================================================================

function ScheduleCard({
  schedule,
  onEdit,
  onDelete,
  onToggle,
}: {
  schedule: ScheduleProfileView;
  onEdit: (s: ScheduleProfileView) => void;
  onDelete: (s: ScheduleProfileView) => void;
  onToggle: (s: ScheduleProfileView) => void;
}) {
  const statusColor = schedule.enabled ? theme.colors.success : theme.colors.textMuted;
  const statusBg = schedule.enabled ? theme.colors.successDim : "rgba(255,255,255,0.04)";

  return (
    <div style={{
      background: theme.colors.panel,
      border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg,
      padding: theme.spacing.lg,
      transition: theme.transition.normal,
    }}>
      {/* Header row */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: theme.spacing.md }}>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "4px" }}>
            <h3 style={{ margin: 0, fontSize: theme.font.sizeLg, fontWeight: 600, color: theme.colors.textPrimary }}>
              {schedule.name}
            </h3>
            <Badge variant={triggerBadgeVariant(schedule.trigger_type)}>{TRIGGER_LABELS[schedule.trigger_type]}</Badge>
          </div>
          <p style={{ margin: "2px 0 0", fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
            {schedule.trigger_summary}
          </p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          {/* Toggle */}
          <button
            onClick={() => onToggle(schedule)}
            title={schedule.enabled ? "Disable" : "Enable"}
            style={{
              display: "flex", alignItems: "center", gap: "4px",
              padding: "4px 10px", border: "1px solid " + (schedule.enabled ? theme.colors.success : theme.colors.panelBorder),
              borderRadius: theme.radius.full, background: statusBg, color: statusColor,
              fontSize: theme.font.sizeXs, fontWeight: 600, cursor: "pointer",
              transition: theme.transition.fast,
            }}
          >
            <PowerIcon />
            {schedule.enabled ? "Enabled" : "Disabled"}
          </button>
          {/* Edit */}
          <button onClick={() => onEdit(schedule)} title="Edit" style={{
            display: "flex", alignItems: "center", justifyContent: "center",
            width: "32px", height: "32px", border: "1px solid " + theme.colors.panelBorder,
            borderRadius: theme.radius.md, background: "transparent", color: theme.colors.textSecondary,
            cursor: "pointer", transition: theme.transition.fast,
          }}>
            <EditIcon />
          </button>
          {/* Delete (disabled if schedule is in use) */}
          <button
            onClick={() => { if (schedule.used_by_count === 0) onDelete(schedule); }}
            title={schedule.used_by_count > 0 ? "In use by " + schedule.used_by_count + " job(s) — cannot delete" : "Delete"}
            disabled={schedule.used_by_count > 0}
            style={{
            display: "flex", alignItems: "center", justifyContent: "center",
            width: "32px", height: "32px",
            border: "1px solid " + (schedule.used_by_count > 0 ? "transparent" : theme.colors.panelBorder),
            borderRadius: theme.radius.md, background: "transparent",
            color: schedule.used_by_count > 0 ? theme.colors.textMuted : theme.colors.textSecondary,
            cursor: schedule.used_by_count > 0 ? "not-allowed" : "pointer",
            opacity: schedule.used_by_count > 0 ? 0.4 : 1,
            transition: theme.transition.fast,
          }}>
            <TrashIcon />
          </button>
        </div>
      </div>

      {/* Detail row */}
      <div style={{ display: "flex", gap: theme.spacing.xl, flexWrap: "wrap", fontSize: theme.font.sizeSm, color: theme.colors.textSecondary }}>
        <div>
          <span style={{ color: theme.colors.textMuted }}>Used by: </span>
          <span style={{ color: theme.colors.textPrimary, fontWeight: 500 }}>
            {schedule.used_by_count > 0 ? schedule.used_by_jobs.join(", ") : "No jobs"}
          </span>
        </div>
        {schedule.next_run_at && (
          <div>
            <span style={{ color: theme.colors.textMuted }}>Next: </span>
            <span style={{ color: theme.colors.textPrimary }}>{schedule.next_run_at}</span>
          </div>
        )}
        {schedule.last_run_at && (
          <div>
            <span style={{ color: theme.colors.textMuted }}>Last: </span>
            <span style={{ color: theme.colors.textPrimary }}>
              {schedule.last_run_status && (
                <span style={{
                  color: schedule.last_run_status === "success" ? theme.colors.success
                    : schedule.last_run_status === "failure" ? theme.colors.error
                    : theme.colors.warning,
                  marginRight: "4px",
                }}>
                  {schedule.last_run_status === "success" ? "✓" : schedule.last_run_status === "failure" ? "✗" : "?"}
                </span>
              )}
              {schedule.last_run_at}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Create/Edit Schedule Form Modal
// ============================================================================

function ScheduleFormModal({
  editing,
  onSave,
  onClose,
}: {
  editing: ScheduleProfileView | null;
  onSave: (req: ScheduleProfileRequest) => void;
  onClose: () => void;
}) {
  const [name, setName] = useState(editing?.name ?? "");
  const [description, setDescription] = useState(editing?.description ?? "");
  const [enabled, setEnabled] = useState(editing?.enabled ?? true);
  const [triggerType, setTriggerType] = useState<TriggerType>(editing?.trigger_type ?? "Daily");
  const [error, setError] = useState<string | null>(null);

  // --- Trigger-type-specific parameter states ---
  // Parsed from editing?.trigger_params on init; combined on save.

  // Daily: time string "21:00"
  const [dailyTime, setDailyTime] = useState("21:00");

  // Weekly: days[] + time string
  const DAY_NAMES = ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"] as const;
  const daySet = new Set(editing?.trigger_params?.split("@")[0]?.split(",") ?? []);
  const [weeklyDays, setWeeklyDays] = useState<string[]>(
    DAY_NAMES.filter(d => daySet.has(d))
  );
  const [weeklyTime, setWeeklyTime] = useState(
    editing?.trigger_params?.split("@")[1] ?? "20:00"
  );

  // Monthly: dayNum + time
  const monthlyParts = editing?.trigger_params?.split("@") ?? [];
  const [monthlyDay, setMonthlyDay] = useState(monthlyParts[0] ?? "15");
  const [monthlyTime, setMonthlyTime] = useState(monthlyParts[1] ?? "03:00");

  // Once: date + time
  // trigger_params format: "2026-08-01 10:00"
  const onceMatch = editing?.trigger_params?.match(/^(\S+)\s+(.+)$/);
  const [onceDate, setOnceDate] = useState(onceMatch?.[1] ?? "2026-08-01");
  const [onceTime, setOnceTime] = useState(onceMatch?.[2] ?? "10:00");

  // OnLogon: delay seconds
  const [onLogonDelay, setOnLogonDelay] = useState(
    editing?.trigger_type === "OnLogon"
      ? (editing?.trigger_params ?? "0")
      : "0"
  );

  const isEditing = editing !== null;

  // Reset per-type defaults when trigger type changes
  const handleTriggerTypeChange = (t: TriggerType) => {
    setTriggerType(t);
    setError(null);
  };

  const buildTriggerParams = (): string => {
    switch (triggerType) {
      case "Daily":   return dailyTime;
      case "Weekly":  return weeklyDays.join(",") + "@" + weeklyTime;
      case "Monthly": return monthlyDay + "@" + monthlyTime;
      case "Once":    return onceDate + " " + onceTime;
      case "OnLogon": return onLogonDelay;
    }
  };

  const handleSave = () => {
    if (!name.trim()) { setError("Schedule name is required"); return; }

    // Validate per-type
    switch (triggerType) {
      case "Weekly":
        if (weeklyDays.length === 0) { setError("Select at least one day of the week"); return; }
        break;
      case "Monthly": {
        const d = parseInt(monthlyDay, 10);
        if (isNaN(d) || d < 1 || d > 31) { setError("Day must be between 1 and 31"); return; }
        break;
      }
      case "OnLogon": {
        const secs = parseInt(onLogonDelay, 10);
        if (isNaN(secs) || secs < 0) { setError("Delay must be a non-negative number"); return; }
        break;
      }
    }

    setError(null);
    onSave({
      name: name.trim(),
      description: description.trim() || null,
      enabled,
      trigger_type: triggerType,
      trigger_params: buildTriggerParams(),
    });
  };

  const inputStyle: React.CSSProperties = {
    width: "100%", padding: "8px 12px",
    background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder,
    borderRadius: theme.radius.md, color: theme.colors.textPrimary,
    fontSize: theme.font.sizeSm, outline: "none", boxSizing: "border-box",
  };

  const labelStyle: React.CSSProperties = {
    display: "block", marginBottom: "4px",
    fontSize: theme.font.sizeXs, fontWeight: 600, color: theme.colors.textMuted,
    textTransform: "uppercase", letterSpacing: "0.5px",
  };

  // Renders sub-controls based on trigger type
  const renderTriggerParamsUI = () => {
    switch (triggerType) {
      case "Daily":
        return (
          <div>
            <label style={{ ...labelStyle, marginTop: "8px" }}>Time</label>
            <input type="time" style={inputStyle} value={dailyTime}
              onChange={(e) => setDailyTime(e.target.value)} />
            <p style={{ margin: "4px 0 0", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
              Set the daily backup time
            </p>
          </div>
        );

      case "Weekly":
        return (
          <div>
            <label style={{ ...labelStyle, marginTop: "8px" }}>Days of Week</label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: "6px", marginBottom: "8px" }}>
              {DAY_NAMES.map((d) => {
                const active = weeklyDays.includes(d);
                return (
                  <button key={d} type="button" onClick={() => {
                    setWeeklyDays(prev =>
                      prev.includes(d) ? prev.filter(x => x !== d) : [...prev, d]
                    );
                  }} style={{
                    padding: "6px 14px", borderRadius: theme.radius.md,
                    border: active ? "1px solid " + theme.colors.primary : "1px solid " + theme.colors.panelBorder,
                    background: active ? theme.colors.primaryDim : theme.colors.panel,
                    color: active ? theme.colors.primary : theme.colors.textSecondary,
                    fontSize: theme.font.sizeSm, fontWeight: active ? 600 : 400,
                    cursor: "pointer", transition: "all 0.15s",
                  }}>
                    {d}
                  </button>
                );
              })}
            </div>
            <label style={labelStyle}>Time</label>
            <input type="time" style={inputStyle} value={weeklyTime}
              onChange={(e) => setWeeklyTime(e.target.value)} />
            <p style={{ margin: "4px 0 0", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
              Select days and set backup time
            </p>
          </div>
        );

      case "Monthly":
        return (
          <div>
            <label style={{ ...labelStyle, marginTop: "8px" }}>Day of Month</label>
            <input type="number" min={1} max={31} style={inputStyle} value={monthlyDay}
              onChange={(e) => setMonthlyDay(e.target.value)}
              placeholder="1-31" />
            <label style={{ ...labelStyle, marginTop: "8px" }}>Time</label>
            <input type="time" style={inputStyle} value={monthlyTime}
              onChange={(e) => setMonthlyTime(e.target.value)} />
            <p style={{ margin: "4px 0 0", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
              Backup runs on day {monthlyDay || "?"} of each month at the set time
            </p>
          </div>
        );

      case "Once":
        return (
          <div>
            <label style={{ ...labelStyle, marginTop: "8px" }}>Date</label>
            <input type="date" style={inputStyle} value={onceDate}
              onChange={(e) => setOnceDate(e.target.value)} />
            <label style={{ ...labelStyle, marginTop: "8px" }}>Time</label>
            <input type="time" style={inputStyle} value={onceTime}
              onChange={(e) => setOnceTime(e.target.value)} />
            <p style={{ margin: "4px 0 0", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
              Backup runs once on the specified date and time
            </p>
          </div>
        );

      case "OnLogon":
        return (
          <div>
            <label style={{ ...labelStyle, marginTop: "8px" }}>Delay (seconds)</label>
            <input type="number" min={0} style={inputStyle} value={onLogonDelay}
              onChange={(e) => setOnLogonDelay(e.target.value)}
              placeholder="0" />
            <p style={{ margin: "4px 0 0", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
              Delay in seconds before backup starts after logon (0 = no delay)
            </p>
          </div>
        );
    }
  };

  return (
    <div style={{
      position: "fixed", inset: 0, zIndex: 1000,
      display: "flex", alignItems: "center", justifyContent: "center",
      background: theme.colors.overlay,
    }} onClick={onClose}>
      <div style={{
        background: theme.colors.panelElevated, border: "1px solid " + theme.colors.panelBorder,
        borderRadius: theme.radius.xl, padding: theme.spacing.xl,
        width: "520px", maxWidth: "90vw", maxHeight: "90vh", overflowY: "auto",
        boxShadow: theme.shadow.elevated,
      }} onClick={(e) => e.stopPropagation()}>
        <h2 style={{ margin: "0 0 " + theme.spacing.lg, fontSize: theme.font.sizeXl, fontWeight: 700, color: theme.colors.textPrimary }}>
          {isEditing ? "Edit Schedule" : "Create Schedule"}
        </h2>

        {error && (
          <div style={{ padding: "8px 12px", marginBottom: theme.spacing.md, background: theme.colors.errorDim, borderRadius: theme.radius.md, fontSize: theme.font.sizeSm, color: theme.colors.error }}>
            {error}
          </div>
        )}

        {/* Name (disabled on edit ? name is the unique identifier) */}
        <div style={{ marginBottom: theme.spacing.md }}>
          <label style={labelStyle}>Name</label>
          <input style={inputStyle} value={name} onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Daily Evening Backup" disabled={isEditing} />
          {isEditing && (
            <p style={{ margin: "4px 0 0", fontSize: theme.font.sizeXs, color: theme.colors.textMuted }}>
              Name cannot be changed after creation. Delete and recreate instead.
            </p>
          )}
        </div>

        {/* Description */}
        <div style={{ marginBottom: theme.spacing.md }}>
          <label style={labelStyle}>Description (optional)</label>
          <input style={inputStyle} value={description} onChange={(e) => setDescription(e.target.value)}
            placeholder="Optional description" />
        </div>

        {/* Trigger Type */}
        <div style={{ marginBottom: theme.spacing.md }}>
          <label style={labelStyle}>Trigger Type</label>
          <select style={inputStyle} value={triggerType} onChange={(e) => handleTriggerTypeChange(e.target.value as TriggerType)}>
            <option value="Once">Once</option>
            <option value="Daily">Daily</option>
            <option value="Weekly">Weekly</option>
            <option value="Monthly">Monthly</option>
            <option value="OnLogon">On Logon</option>
          </select>
        </div>

        {/* Trigger Params -- dynamic UI per type */}
        {renderTriggerParamsUI()}

        {/* Enabled */}
        <div style={{ marginBottom: theme.spacing.lg, display: "flex", alignItems: "center", gap: "8px" }}>
          <input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)}
            style={{ accentColor: theme.colors.primary }} />
          <label style={{ fontSize: theme.font.sizeSm, color: theme.colors.textPrimary }}>Enable after creation</label>
        </div>

        {/* Actions */}
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
          <Button variant="secondary" onClick={onClose}>Cancel</Button>
          <Button variant="primary" onClick={handleSave}>{isEditing ? "Save Changes" : "Create Schedule"}</Button>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Delete Confirmation Dialog
// ============================================================================

function DeleteConfirmModal({
  schedule,
  onConfirm,
  onClose,
  loading,
}: {
  schedule: ScheduleProfileView;
  onConfirm: () => void;
  onClose: () => void;
  loading: boolean;
}) {
  return (
    <div style={{
      position: "fixed", inset: 0, zIndex: 1000,
      display: "flex", alignItems: "center", justifyContent: "center",
      background: theme.colors.overlay,
    }} onClick={onClose}>
      <div style={{
        background: theme.colors.panelElevated, border: "1px solid " + theme.colors.panelBorder,
        borderRadius: theme.radius.xl, padding: theme.spacing.xl,
        width: "440px", maxWidth: "90vw",
        boxShadow: theme.shadow.elevated,
      }} onClick={(e) => e.stopPropagation()}>
        <h3 style={{ margin: "0 0 " + theme.spacing.sm, fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary }}>
          Delete Schedule
        </h3>
        <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, margin: "0 0 " + theme.spacing.md }}>
          Are you sure you want to delete <strong style={{ color: theme.colors.textPrimary }}>{schedule.name}</strong>?
        </p>

        <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
          <Button variant="secondary" onClick={onClose} disabled={loading}>Cancel</Button>
          <Button variant="danger" onClick={onConfirm} disabled={loading}>
            {loading ? "Deleting..." : "Delete Schedule"}
          </Button>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Main Schedule Page
// ============================================================================

export default function Schedule() {
  const [schedules, setSchedules] = useState<ScheduleProfileView[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Modal states
  const [formOpen, setFormOpen] = useState(false);
  const [editingSchedule, setEditingSchedule] = useState<ScheduleProfileView | null>(null);

  // Delete confirmation (only shown for unused schedules)
  const [deletingSchedule, setDeletingSchedule] = useState<ScheduleProfileView | null>(null);
  const [deleteLoading, setDeleteLoading] = useState(false);

  // ---- Data loading ----
  const fetchSchedules = useCallback(() => {
    setLoading(true);
    setError(null);
    listSchedules()
      .then(setSchedules)
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : "Failed to load schedules";
        setError(msg);
      })
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => { fetchSchedules(); }, [fetchSchedules]);

  // ---- CRUD handlers ----
  const handleCreate = () => {
    setEditingSchedule(null);
    setFormOpen(true);
  };

  const handleEdit = (s: ScheduleProfileView) => {
    setEditingSchedule(s);
    setFormOpen(true);
  };

  const handleFormSave = (req: ScheduleProfileRequest) => {
    const action = editingSchedule
      ? updateSchedule(editingSchedule.id, req)
      : createSchedule(req);
    action
      .then(() => {
        setFormOpen(false);
        setEditingSchedule(null);
        fetchSchedules();
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : "Failed to save schedule";
        setError(msg);
      });
  };

  const handleDeleteClick = (s: ScheduleProfileView) => {
    // Show confirmation modal (schedule is not in use — button disabled if used_by_count > 0)
    setDeletingSchedule(s);
  };

  const handleDeleteConfirm = () => {
    if (!deletingSchedule) return;
    setDeleteLoading(true);
    deleteSchedule(deletingSchedule.id, true)
      .then(() => {
        setDeletingSchedule(null);
        fetchSchedules();
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : "Failed to delete schedule";
        setError(msg);
        setDeleteLoading(false);
      });
  };

  const handleToggle = (s: ScheduleProfileView) => {
    const action = s.enabled ? disableSchedule(s.id) : enableSchedule(s.id);
    action
      .then(() => fetchSchedules())
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : "Failed to toggle schedule";
        setError(msg);
      });
  };

  // ---- Loading ----
  if (loading) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <div style={{ marginBottom: "24px" }}>
          <div style={{ width: "200px", height: "28px", borderRadius: theme.radius.sm, background: "linear-gradient(90deg, #1e2a45 25%, #253050 50%, #1e2a45 75%)", backgroundSize: "200% 100%", animation: "skeletonPulse 1.5s infinite" }} />
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(480px, 1fr))", gap: theme.spacing.md }}>
          {[0, 1, 2].map((i) => <SkeletonCard key={i} />)}
        </div>
      </div>
    );
  }

  // ---- Error ----
  if (error && schedules.length === 0) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <ErrorState title="Failed to load schedules" message={error} onRetry={fetchSchedules} />
      </div>
    );
  }

  // ---- Empty ----
  if (schedules.length === 0) {
    return (
      <div style={{ padding: theme.spacing.lg }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "24px" }}>
          <div>
            <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>Schedule</h2>
            <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>
              Create schedules to automate backup jobs
            </p>
          </div>
          <Button variant="primary" size="sm" onClick={handleCreate} icon={<PlusIcon />}>Create Schedule</Button>
        </div>
        <EmptyState
          title="No Schedules"
          description="Create a schedule to automatically run backup jobs on a daily, weekly, or monthly basis."
        />
        {formOpen && (
          <ScheduleFormModal editing={null} onSave={handleFormSave} onClose={() => { setFormOpen(false); setEditingSchedule(null); }} />
        )}
      </div>
    );
  }

  // ---- Data ----
  return (
    <div style={{ padding: theme.spacing.lg }}>
      {/* Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "24px" }}>
        <div>
          <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>Schedule</h2>
          <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>
            {schedules.length} schedule{schedules.length !== 1 ? "s" : ""} configured
          </p>
        </div>
        <div style={{ display: "flex", gap: "8px" }}>
          <Button variant="secondary" size="sm" icon={<RefreshIcon />} onClick={fetchSchedules}>Refresh</Button>
          <Button variant="primary" size="sm" icon={<PlusIcon />} onClick={handleCreate}>Create Schedule</Button>
        </div>
      </div>

      {/* Grid */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(480px, 1fr))", gap: theme.spacing.md }}>
        {schedules.map((s) => (
          <ScheduleCard key={s.id} schedule={s} onEdit={handleEdit} onDelete={handleDeleteClick} onToggle={handleToggle} />
        ))}
      </div>

      {/* Create/Edit Modal */}
      {formOpen && (
        <ScheduleFormModal
          editing={editingSchedule}
          onSave={handleFormSave}
          onClose={() => { setFormOpen(false); setEditingSchedule(null); }}
        />
      )}

      {/* Delete Confirmation */}
      {deletingSchedule && !deleteLoading && (
        <DeleteConfirmModal
          schedule={deletingSchedule}
          onConfirm={handleDeleteConfirm}
          onClose={() => { setDeletingSchedule(null); }}
          loading={deleteLoading}
        />
      )}
    </div>
  );
}
