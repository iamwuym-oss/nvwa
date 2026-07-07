import { theme } from "../../theme";

interface ButtonProps {
  variant?: "primary" | "secondary" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  children: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  icon?: React.ReactNode;
  style?: React.CSSProperties;
}

const btnBase: React.CSSProperties = {
  display: "inline-flex", alignItems: "center", gap: "8px",
  border: "none", borderRadius: theme.radius.md, fontWeight: 600,
  cursor: "pointer", transition: theme.transition.fast,
  whiteSpace: "nowrap", fontFamily: theme.font.family,
};

const variants: Record<string, React.CSSProperties> = {
  primary:  { background: theme.colors.primary, color: "#ffffff" },
  secondary: { background: "rgba(255,255,255,0.06)", color: theme.colors.textPrimary, border: "1px solid rgba(255,255,255,0.1)" },
  ghost:    { background: "transparent", color: theme.colors.textSecondary },
  danger:   { background: theme.colors.errorDim, color: theme.colors.error, border: "1px solid rgba(239,68,68,0.2)" },
};

const sizes: Record<string, React.CSSProperties> = {
  sm: { padding: "5px 12px", fontSize: "12px", borderRadius: "6px" },
  md: { padding: "8px 18px", fontSize: "14px" },
  lg: { padding: "12px 24px", fontSize: "15px" },
};

export default function Button({ variant = "primary", size = "md", children, onClick, disabled, icon, style }: ButtonProps) {
  return (
    <button style={{ ...btnBase, ...variants[variant], ...sizes[size], ...style }} onClick={onClick} disabled={disabled}>
      {icon && <span>{icon}</span>}{children}
    </button>
  );
}
