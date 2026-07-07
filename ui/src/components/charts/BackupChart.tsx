import { theme } from "../../theme";

interface DataPoint { label: string; value: number }

interface BackupChartProps {
  data: DataPoint[];
  height?: number;
  color?: string;
}

export default function BackupChart({ data, height = 150, color = theme.colors.primary }: BackupChartProps) {
  const max = Math.max(...data.map(d => d.value), 1);
  return (
    <div style={{
      background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg, padding: "18px 20px",
    }}>
      <h3 style={{
        fontSize: "13px", fontWeight: 600, color: theme.colors.textSecondary,
        textTransform: "uppercase", letterSpacing: "0.5px", margin: "0 0 16px 0",
      }}>
        Backup Overview (Last 7 Days)
      </h3>
      <div style={{ display: "flex", alignItems: "flex-end", gap: "6px", height: height + "px", position: "relative" }}>
        {[0,1,2,3].map(i => (
          <div key={i} style={{
            position: "absolute", left: 0, right: 0,
            top: (height * i / 4) + "px",
            borderTop: "1px solid rgba(255,255,255,0.04)",
          }} />
        ))}
        {data.map((d, i) => {
          const pct = Math.max((d.value / max) * 100, 4);
          return (
            <div key={i} style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", height: "100%", justifyContent: "flex-end" }}>
              <div style={{
                width: "24px", height: pct + "%",
                background: "linear-gradient(180deg, " + color + " 0%, rgba(22,131,255,0.3) 100%)",
                borderRadius: "4px 4px 0 0", transition: theme.transition.slow,
              }} />
              <span style={{ fontSize: "10px", color: theme.colors.textMuted, whiteSpace: "nowrap", marginTop: "4px" }}>{d.label}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
