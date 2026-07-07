// ============================================================================
// EmptyState.tsx -- Reusable empty-state placeholder
//
// Used when the Dashboard has no configured backup jobs.
// Provides a clear message and a call-to-action button.
// ============================================================================

import { theme } from "../../theme";
import Button from "../common/Button";

interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description: string;
  primaryAction?: { label: string; onClick?: () => void; disabled?: boolean };
  secondaryAction?: { label: string; onClick?: () => void; disabled?: boolean };
}

export default function EmptyState({
  icon,
  title,
  description,
  primaryAction,
  secondaryAction,
}: EmptyStateProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        padding: "80px 40px",
        textAlign: "center",
      }}
    >
      {/* Icon */}
      {icon && (
        <div
          style={{
            width: "80px",
            height: "80px",
            borderRadius: "50%",
            background: theme.colors.primaryDim,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: "36px",
            color: theme.colors.primary,
            marginBottom: "24px",
            boxShadow: `0 0 40px ${theme.colors.primaryGlow}`,
          }}
        >
          {icon}
        </div>
      )}

      {/* Title */}
      <h2
        style={{
          fontSize: theme.font.sizeXl,
          fontWeight: 700,
          color: theme.colors.textPrimary,
          margin: "0 0 8px 0",
          lineHeight: 1.3,
        }}
      >
        {title}
      </h2>

      {/* Description */}
      <p
        style={{
          fontSize: theme.font.sizeMd,
          color: theme.colors.textSecondary,
          margin: "0 0 28px 0",
          lineHeight: 1.6,
          maxWidth: "480px",
        }}
      >
        {description}
      </p>

      {/* Actions */}
      {(primaryAction || secondaryAction) && (
        <div style={{ display: "flex", gap: "12px" }}>
          {primaryAction && (
            <Button
              variant="primary"
              size="lg"
              onClick={primaryAction.onClick}
              disabled={primaryAction.disabled}
            >
              {primaryAction.label}
            </Button>
          )}
          {secondaryAction && (
            <Button
              variant="secondary"
              size="lg"
              onClick={secondaryAction.onClick}
              disabled={secondaryAction.disabled}
            >
              {secondaryAction.label}
            </Button>
          )}
        </div>
      )}
    </div>
  );
}
