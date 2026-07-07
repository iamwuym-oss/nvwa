// ============================================================================
// ErrorState.tsx -- Error display with retry and recovery suggestions
//
// When the Dashboard fails to load (config missing, storage down, etc.),
// this component shows a clear error card with possible reasons and a retry
// button instead of a raw error message.
// ============================================================================

import { theme } from "../../theme";
import Button from "../common/Button";

interface ErrorStateProps {
  title?: string;
  message: string;
  reasons?: string[];
  onRetry?: () => void;
}

export default function ErrorState({
  title = "Dashboard unavailable",
  message,
  reasons,
  onRetry,
}: ErrorStateProps) {
  return (
    <div
      style={{
        maxWidth: "520px",
        margin: "60px auto",
        padding: "36px 32px",
        background: theme.colors.errorDim,
        border: `1px solid ${theme.colors.error}30`,
        borderRadius: theme.radius.xl,
        textAlign: "center",
      }}
    >
      {/* Warning icon */}
      <div
        style={{
          width: "56px",
          height: "56px",
          borderRadius: "50%",
          background: `${theme.colors.error}20`,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          margin: "0 auto 18px auto",
          fontSize: "28px",
          color: theme.colors.error,
        }}
      >
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
      </div>

      {/* Error title */}
      <h3
        style={{
          fontSize: theme.font.sizeLg,
          fontWeight: 700,
          color: theme.colors.textPrimary,
          margin: "0 0 8px 0",
        }}
      >
        {title}
      </h3>

      {/* Error message */}
      <p
        style={{
          fontSize: theme.font.sizeMd,
          color: theme.colors.textSecondary,
          margin: "0 0 18px 0",
          lineHeight: 1.6,
        }}
      >
        {message}
      </p>

      {/* Possible reasons */}
      {reasons && reasons.length > 0 && (
        <div
          style={{
            textAlign: "left",
            margin: "0 0 20px 0",
            padding: "12px 16px",
            background: "rgba(0,0,0,0.2)",
            borderRadius: theme.radius.md,
          }}
        >
          <div
            style={{
              fontSize: theme.font.sizeXs,
              fontWeight: 600,
              color: theme.colors.textMuted,
              textTransform: "uppercase",
              letterSpacing: "0.5px",
              marginBottom: "8px",
            }}
          >
            Possible reasons
          </div>
          <ul
            style={{
              margin: 0,
              padding: "0 0 0 16px",
              fontSize: theme.font.sizeSm,
              color: theme.colors.textSecondary,
              lineHeight: 1.8,
            }}
          >
            {reasons.map((r, i) => (
              <li key={i}>{r}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Retry button */}
      {onRetry && (
        <Button variant="secondary" size="md" onClick={onRetry}>
          Retry
        </Button>
      )}
    </div>
  );
}
