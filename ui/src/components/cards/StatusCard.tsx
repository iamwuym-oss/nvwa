import { theme } from "../../theme";

interface StatusCardProps {
  icon: React.ReactNode;
  title: string;
  status: "healthy" | "warning" | "critical" | "ok" | "warn" | "fail";
  details: { label: string; value: string }[];
}

const scMap: Record<string, { dot: string; bg: string; label: string }> = {
  healthy: { dot: theme.colors.success, bg: theme.colors.successDim, label: "Healthy" },
  ok:      { dot: theme.colors.success, bg: theme.colors.successDim, label: "OK" },
  warning: { dot: theme.colors.warning, bg: theme.colors.warningDim, label: "Warning" },
  warn:    { dot: theme.colors.warning, bg: theme.colors.warningDim, label: "Warning" },
  critical:{ dot: theme.colors.error, bg: theme.colors.errorDim, label: "Critical" },
  fail:    { dot: theme.colors.error, bg: theme.colors.errorDim, label: "Failed" },
};

export default function StatusCard({ icon, title, status, details }: StatusCardProps) {
  const s = scMap[status] || scMap.healthy;
  return (
    <div style={{
      background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg, padding: "18px 20px",
    }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "14px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <span style={{ fontSize: "16px", color: theme.colors.textSecondary }}>{icon}</span>
          <span style={{ fontSize: "13px", fontWeight: 600, color: theme.colors.textPrimary }}>{title}</span>
        </div>
        <span style={{
          display: "inline-flex", alignItems: "center", gap: "5px",
          fontSize: "11px", fontWeight: 600, color: s.dot,
          padding: "2px 8px", borderRadius: theme.radius.full, background: s.bg,
        }}>
          <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: s.dot }} />
          {s.label}
        </span>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
        {details.map((d, i) => (
          <div key={i} style={{ display: "flex", justifyContent: "space-between", fontSize: "12px" }}>
            <span style={{ color: theme.colors.textMuted }}>{d.label}</span>
            <span style={{ color: theme.colors.textSecondary }}>{d.value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
