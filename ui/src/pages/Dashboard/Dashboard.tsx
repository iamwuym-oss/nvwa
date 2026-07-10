// ============================================================================
// Dashboard.tsx -- Dashboard page
//
// Data flow:
//   useEffect -> getDashboard() -> DashboardView -> render
//
// States handled:
//   - Loading: full skeleton layout that mirrors the card grid
//   - Error:   error card with reasons + retry button
//   - Empty:   welcome screen with CTA when no backup jobs exist
//   - Data:    full dashboard with HeroCard, MetricCards, charts, activity
// ============================================================================

import React, { useEffect, useState, useCallback } from "react";
import HeroCard from "../../components/cards/HeroCard";
import MetricCard from "../../components/cards/MetricCard";
import StatusCard from "../../components/cards/StatusCard";
import BackupChart from "../../components/charts/BackupChart";
import Panel from "../../components/common/Panel";
import Badge from "../../components/common/Badge";
import LoadingState from "../../components/feedback/LoadingState";
import EmptyState from "../../components/feedback/EmptyState";
import ErrorState from "../../components/feedback/ErrorState";
import NoActivityState from "../../components/feedback/NoActivityState";
import { theme } from "../../theme";
import { getDashboard, DashboardView } from "../../api/dashboardApi";
import type { Page } from "../../App";

// ---------------------------------------------------------------------------
// Chart data (static for now, will come from backend in a future phase)
// ---------------------------------------------------------------------------
const chartData = [
  { label: "01", value: 120 }, { label: "02", value: 95 },
  { label: "03", value: 145 }, { label: "04", value: 110 },
  { label: "05", value: 160 }, { label: "06", value: 135 },
  { label: "07", value: 0 },
];

// ---------------------------------------------------------------------------
// Inline SVG icon helpers
// ---------------------------------------------------------------------------
function ShieldIcon() {
  return React.createElement("svg", { width: 20, height: 20, viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: 2 },
    React.createElement("path", { d: "M12 2L3 7v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-9-5z" })
  );
}
function ClockIcon() {
  return React.createElement("svg", { width: 20, height: 20, viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: 2 },
    React.createElement("circle", { cx: 12, cy: 12, r: 10 }),
    React.createElement("path", { d: "M12 6v6l4 2" })
  );
}
function StorageIcon() {
  return React.createElement("svg", { width: 20, height: 20, viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: 2 },
    React.createElement("ellipse", { cx: 12, cy: 5, rx: 9, ry: 3 }),
    React.createElement("path", { d: "M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" }),
    React.createElement("path", { d: "M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" })
  );
}
function HeartIcon() {
  return React.createElement("svg", { width: 20, height: 20, viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: 2 },
    React.createElement("path", { d: "M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z" })
  );
}
function ShieldLargeIcon() {
  return React.createElement("svg", { width: 36, height: 36, viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: 1.5 },
    React.createElement("path", { d: "M12 2L3 7v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-9-5z" }),
    React.createElement("path", { d: "M9 12l2 2 4-4", stroke: theme.colors.success })
  );
}

// ---------------------------------------------------------------------------
// Status badge for activity items
// ---------------------------------------------------------------------------
function StatusBadge({ status }: { status: string }) {
  const m: Record<string, { variant: "success" | "warning" | "error"; char: string }> = {
    success: { variant: "success", char: "\u2713" },
    warning: { variant: "warning", char: "\u26A0" },
    error: { variant: "error", char: "\u2717" },
  };
  const match = m[status] || { variant: "info" as const, char: "\u2139" };
  return React.createElement(Badge, { variant: match.variant }, match.char);
}

// ---------------------------------------------------------------------------
export default function Dashboard({ onNavigate }: { onNavigate?: (page: Page) => void }) {
  const [data, setData] = useState<DashboardView | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    getDashboard()
      .then((result) => {
        if (!cancelled) {
          setData(result);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err?.message || "Failed to load dashboard data");
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(fetchData, [fetchData]);

  // ---- Loading State (skeleton) ----
  if (loading) {
    return <LoadingState />;
  }

  // ---- Error State ----
  if (error || !data) {
    return (
      <ErrorState
        message={error || "An unexpected error occurred while loading the dashboard."}
        reasons={[
          "Configuration file is missing or corrupt",
          "Storage drive is not accessible",
          "Permission denied when reading backup data",
        ]}
        onRetry={fetchData}
      />
    );
  }

  // ---- Empty State (no backup jobs configured) ----
  if (data.totalJobs === 0) {
    return (
      <EmptyState
        icon={React.createElement(ShieldLargeIcon)}
        title="No Backup Plan Yet"
        description="Create your first backup plan to protect your important data from accidental loss or system failure."
        primaryAction={{ label: "Create Backup", disabled: true }}
        secondaryAction={{ label: "Import Configuration", disabled: true }}
      />
    );
  }

  // ---- Data State ----
  const hasHistory = data.recentActivity.length > 0;

  return (
    <div style={{ maxWidth: "1200px", margin: "0 auto" }}>
      {/* ---- Hero Section ---- */}
      <div style={{ marginBottom: "20px" }}>
        <HeroCard protectionStatus={data.protectionStatus} healthStatus={data.healthStatus} totalJobs={data.totalJobs} lastBackupAgo={data.lastBackupAgo} onBackupNow={onNavigate ? () => onNavigate("backup") : undefined} onRestoreFiles={onNavigate ? () => onNavigate("restore") : undefined} />
      </div>

      {/* ---- Metric Cards ---- */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(4, 1fr)",
          gap: "16px",
          marginBottom: "20px",
        }}
      >
        <div className="dashboard-card">
          <MetricCard
            icon={React.createElement(ShieldIcon)}
            title="Protected Jobs"
            value={data.protectedJobs + "/" + data.totalJobs}
            subtitle="Active protection"
            color={
              data.protectionStatus === "Protected"
                ? theme.colors.success
                : data.protectionStatus === "AtRisk"
                  ? theme.colors.warning
                  : theme.colors.error
            }
            trend={data.protectionStatus === "Protected" ? "up" : "down"}
            trendValue={
              data.protectionStatus === "Protected"
                ? "+" + data.protectedJobs
                : "" + (data.totalJobs - data.protectedJobs)
            }
          />
        </div>
        <div className="dashboard-card">
          <MetricCard
            icon={React.createElement(ClockIcon)}
            title="Last Backup"
            value={data.lastBackupAgo}
            subtitle={data.lastBackupName}
            color={theme.colors.primary}
          />
        </div>
        <div className="dashboard-card">
          <MetricCard
            icon={React.createElement(StorageIcon)}
            title="Storage Used"
            value={data.storageUsed}
            subtitle={"Of " + data.storageTotal}
            color={
              data.storagePercent > 85
                ? theme.colors.error
                : data.storagePercent > 60
                  ? theme.colors.warning
                  : theme.colors.primary
            }
            trend={data.storagePercent > 85 ? "up" : "neutral"}
            trendValue={data.storagePercent.toFixed(0) + "%"}
          />
        </div>
        <div className="dashboard-card">
          <MetricCard
            icon={React.createElement(HeartIcon)}
            title="Health Status"
            value={data.healthStatus}
            subtitle={data.healthDetail}
            color={
              data.healthStatus === "Healthy"
                ? theme.colors.success
                : data.healthStatus === "Warning"
                  ? theme.colors.warning
                  : theme.colors.error
            }
          />
        </div>
      </div>

      {/* ---- Chart + Recent Activity ---- */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "2fr 1fr",
          gap: "16px",
          marginBottom: "20px",
        }}
      >
        <div className="dashboard-card">
          <BackupChart data={chartData} />
        </div>
        <div
          className="dashboard-card"
          style={{
            background: theme.colors.panel,
            border: "1px solid " + theme.colors.panelBorder,
            borderRadius: theme.radius.lg,
            padding: "18px 20px",
            maxHeight: "280px",
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
        >
          <h3
            style={{
              fontSize: "13px",
              fontWeight: 600,
              color: theme.colors.textSecondary,
              textTransform: "uppercase",
              letterSpacing: "0.5px",
              margin: "0 0 12px 0",
              flexShrink: 0,
            }}
          >
            Recent Activity
          </h3>

          {hasHistory ? (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "4px",
                overflowY: "auto",
                flex: 1,
              }}
            >
              {data.recentActivity.map((a) => (
                <div
                  key={a.id}
                  style={{
                    display: "flex",
                    alignItems: "flex-start",
                    gap: "10px",
                    padding: "8px 0",
                    borderBottom: "1px solid " + theme.colors.divider,
                    transition: theme.transition.fast,
                  }}
                  className="activity-row"
                >
                  <div style={{ flexShrink: 0 }}>
                    {React.createElement(StatusBadge, { status: a.status })}
                  </div>
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div
                      style={{
                        fontSize: "12px",
                        color: theme.colors.textPrimary,
                        whiteSpace: "nowrap",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                      }}
                    >
                      {a.message}
                    </div>
                    <div
                      style={{
                        fontSize: "11px",
                        color: theme.colors.textMuted,
                        marginTop: "2px",
                      }}
                    >
                      {a.timestamp}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div style={{ flex: 1, display: "flex", alignItems: "center" }}>
              <NoActivityState />
            </div>
          )}
        </div>
      </div>

      {/* ---- Bottom Panels ---- */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: "16px",
        }}
      >
        {/* Scheduled Jobs */}
        <div className="dashboard-card">
          <Panel
            title="Scheduled Jobs"
            subtitle={data.scheduledCount + " active"}
          >
            {data.scheduledJobs.length > 0
              ? data.scheduledJobs.slice(0, 3).map((j, i) => (
                  <div
                    key={"sched-" + i}
                    style={{
                      display: "flex",
                      justifyContent: "space-between",
                      alignItems: "center",
                      padding: "8px 0",
                      borderBottom: "1px solid " + theme.colors.divider,
                    }}
                  >
                    <div>
                      <div
                        style={{
                          fontSize: "13px",
                          fontWeight: 500,
                          color: theme.colors.textPrimary,
                        }}
                      >
                        {j.name}
                      </div>
                      <div
                        style={{
                          fontSize: "11px",
                          color: theme.colors.textMuted,
                        }}
                      >
                        {j.schedule}
                      </div>
                    </div>
                    <Badge variant="info">{j.nextRun}</Badge>
                  </div>
                ))
              : data.scheduledCount === 0
                ? null
                : null}
            {data.scheduledCount === 0 && (
              <div
                style={{
                  fontSize: "12px",
                  color: theme.colors.textMuted,
                  padding: "16px 0",
                  textAlign: "center",
                }}
              >
                No scheduled jobs configured
              </div>
            )}
          </Panel>
        </div>

        {/* Storage Targets */}
        <div className="dashboard-card">
          <Panel title="Storage Targets">
            {data.storagePercent > 0 ? (
              <StatusCard
                icon={React.createElement(StorageIcon)}
                title="Local Storage"
                status={
                  data.storagePercent > 85
                    ? "critical"
                    : data.storagePercent > 60
                      ? "warning"
                      : "ok"
                }
                details={[
                  { label: "Capacity", value: data.storageTotal },
                  { label: "Used", value: data.storageUsed },
                  { label: "Usage", value: data.storagePercent.toFixed(1) + "%" },
                ]}
              />
            ) : (
              <div
                style={{
                  fontSize: "12px",
                  color: theme.colors.textMuted,
                  padding: "16px 0",
                  textAlign: "center",
                }}
              >
                No storage data available
              </div>
            )}
          </Panel>
        </div>

        {/* Protection Summary */}
        <div className="dashboard-card">
          <Panel title="Protection Summary">
            <StatusCard
              icon={React.createElement(ShieldIcon)}
              title="System Health"
              status={
                data.healthStatus === "Healthy"
                  ? "healthy"
                  : data.healthStatus === "Warning"
                    ? "warning"
                    : "critical"
              }
              details={[
                { label: "Protection", value: data.protectionStatus },
                {
                  label: "Jobs",
                  value: data.protectedJobs + "/" + data.totalJobs + " protected",
                },
                {
                  label: "Scheduled",
                  value: data.scheduledCount + " tasks",
                },
                { label: "Last Backup", value: data.lastBackupAgo },
              ]}
            />
          </Panel>
        </div>
      </div>
    </div>
  );
}



