import { theme } from "../../theme";

interface PanelProps {
  title?: string;
  subtitle?: string;
  children: React.ReactNode;
  style?: React.CSSProperties;
  actions?: React.ReactNode;
}

export default function Panel({ title, subtitle, children, style, actions }: PanelProps) {
  return (
    <div style={{
      background: theme.colors.panel, border: "1px solid " + theme.colors.panelBorder,
      borderRadius: theme.radius.lg, padding: theme.spacing.lg,
      boxShadow: theme.shadow.card, ...style,
    }}>
      {(title || actions) && (
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: subtitle ? "4px" : "16px" }}>
          <div>
            {title && <h3 style={{ fontSize: theme.font.sizeMd, fontWeight: 600, color: theme.colors.textPrimary, margin: 0 }}>{title}</h3>}
            {subtitle && <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, margin: "2px 0 0 0" }}>{subtitle}</p>}
          </div>
          {actions && <div style={{ display: "flex", gap: "8px" }}>{actions}</div>}
        </div>
      )}
      {children}
    </div>
  );
}
