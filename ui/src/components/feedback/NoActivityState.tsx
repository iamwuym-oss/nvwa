// ============================================================================
// NoActivityState.tsx -- Inline empty-state for the Recent Activity panel
//
// Shown when the history database has no records yet.
// ============================================================================

import { theme } from "../../theme";

export default function NoActivityState() {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        padding: "36px 20px",
        textAlign: "center",
      }}
    >
      {/* Clock icon */}
      <div
        style={{
          width: "40px",
          height: "40px",
          borderRadius: "50%",
          background: theme.colors.primaryDim,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontSize: "18px",
          color: theme.colors.primary,
          marginBottom: "12px",
        }}
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
      </div>

      <p
        style={{
          fontSize: theme.font.sizeMd,
          fontWeight: 500,
          color: theme.colors.textSecondary,
          margin: "0 0 4px 0",
        }}
      >
        No backup activity yet
      </p>

      <p
        style={{
          fontSize: theme.font.sizeSm,
          color: theme.colors.textMuted,
          margin: 0,
          lineHeight: 1.5,
          maxWidth: "260px",
        }}
      >
        Run your first backup to see protection history here.
      </p>
    </div>
  );
}
