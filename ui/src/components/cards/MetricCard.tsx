import { theme } from "../../theme";

interface MetricCardProps {
  icon: React.ReactNode;
  title: string;
  value: string;
  subtitle?: string;
  color?: string;
  trend?: "up" | "down" | "neutral";
  trendValue?: string;
}

export default function MetricCard({ icon, title, value, subtitle, color = theme.colors.primary, trend, trendValue }: MetricCardProps) {
  return (
    <div style={{
      background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg, padding: "20px", display: "flex",
      flexDirection: "column", gap: "12px", cursor: "default",
    }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
        <div style={{
          width: "40px", height: "40px", borderRadius: theme.radius.md,
          background: color + "18", display: "flex", alignItems: "center",
          justifyContent: "center", fontSize: "18px", color: color,
        }}>{icon}</div>
        {trend && <span style={{ fontSize: "12px", color: trend === "up" ? theme.colors.success : trend === "down" ? theme.colors.error : theme.colors.textMuted, fontWeight: 600 }}>
          {trend === "up" ? "\u2191" : trend === "down" ? "\u2193" : "\u2192"} {trendValue}
        </span>}
      </div>
      <div>
        <div style={{ fontSize: "22px", fontWeight: 700, color: theme.colors.textPrimary, lineHeight: 1.1 }}>{value}</div>
        <div style={{ fontSize: "13px", color: theme.colors.textSecondary, marginTop: "4px" }}>{title}</div>
        {subtitle && <div style={{ fontSize: "11px", color: theme.colors.textMuted, marginTop: "2px" }}>{subtitle}</div>}
      </div>
    </div>
  );
}
