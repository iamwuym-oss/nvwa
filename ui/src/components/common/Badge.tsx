import { theme } from "../../theme";

interface BadgeProps {
  variant?: "success" | "warning" | "error" | "info" | "neutral";
  children?: React.ReactNode;
  style?: React.CSSProperties;
}

const bc: Record<string, { bg: string; fg: string }> = {
  success: { bg: theme.colors.successDim, fg: theme.colors.success },
  warning: { bg: theme.colors.warningDim, fg: theme.colors.warning },
  error:   { bg: theme.colors.errorDim, fg: theme.colors.error },
  info:    { bg: theme.colors.primaryDim, fg: theme.colors.primary },
  neutral: { bg: "rgba(255,255,255,0.06)", fg: theme.colors.textSecondary },
};

export default function Badge({ variant = "neutral", children, style }: BadgeProps) {
  const c = bc[variant];
  return (
    <span style={{ display: "inline-flex", alignItems: "center", gap: "4px", padding: "2px 8px", borderRadius: theme.radius.full, fontSize: theme.font.sizeXs, fontWeight: 600, background: c.bg, color: c.fg, ...style }}>
      {children}
    </span>
  );
}
