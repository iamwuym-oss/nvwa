// ============================================================================
// Settings.tsx -- Settings page (future: global application settings)
//
// Backup plan configuration has been moved to the Backup page
// (three-column layout: list | right-side config panel).
//
// This page is reserved for:
//   - Global application configuration
//   - Environment configuration
//   - Theme / language / other preferences
// ============================================================================

import { theme } from "../theme";

export default function Settings() {
  return (
    <div style={{ padding: theme.spacing.lg, maxWidth: "700px" }}>
      <div style={{ marginBottom: "24px" }}>
        <h2 style={{ fontSize: theme.font.sizeXxl, fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>
          Settings
        </h2>
        <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "2px" }}>
          Global application configuration
        </p>
      </div>

      <div style={{
        background: theme.colors.panel,
        border: "1px solid " + theme.colors.panelBorder,
        borderRadius: theme.radius.xl,
        padding: theme.spacing.xl,
        textAlign: "center",
      }}>
        <div style={{
          width: "48px", height: "48px", borderRadius: "50%",
          background: theme.colors.primaryDim,
          display: "flex", alignItems: "center", justifyContent: "center",
          margin: "0 auto 16px auto",
        }}>
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke={theme.colors.primary} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
          </svg>
        </div>
        <h3 style={{ fontSize: theme.font.sizeLg, fontWeight: 700, color: theme.colors.textPrimary, margin: "0 0 8px 0" }}>
          Global Settings
        </h3>
        <p style={{ fontSize: theme.font.sizeMd, color: theme.colors.textSecondary, margin: 0, lineHeight: 1.6 }}>
          Backup plan configuration has been moved to the <strong>Backup</strong> page.
        </p>
        <p style={{ fontSize: theme.font.sizeSm, color: theme.colors.textMuted, marginTop: "12px", lineHeight: 1.5 }}>
          This page will be used for global application configuration in a future update.
        </p>
      </div>
    </div>
  );
}
