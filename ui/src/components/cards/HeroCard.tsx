// ============================================================================
// HeroCard.tsx -- Dashboard hero banner showing protection status
//
// Displays a gradient card with the current protection state, status message,
// and quick-action buttons. The content changes based on the actual
// protection_status and health from the backend.
// ============================================================================

import { theme } from "../../theme";
import Button from "../common/Button";
import { ProtectionStatus, HealthStatus } from "../../api/dashboardApi";

interface HeroCardProps {
  protectionStatus: ProtectionStatus;
  healthStatus: HealthStatus;
  totalJobs: number;
  lastBackupAgo: string;
  onBackupNow?: () => void;
  onRestoreFiles?: () => void;
}

// Map protection status to display content
function getHeroContent(status: ProtectionStatus, health: HealthStatus, totalJobs: number, lastBackup: string) {
  switch (status) {
    case "Protected":
      return {
        title: "Your data is protected",
        subtitle:
          health === "Healthy"
            ? `All ${totalJobs} backup job${totalJobs > 1 ? "s are" : " is"} running normally. Last backup completed ${lastBackup}.`
            : "Some systems need attention despite recent backup success.",
        iconColor: "#22c55e",
        iconCheck: true,
        borderColor: "rgba(34, 197, 94, 0.15)",
        glowColor: "rgba(34, 197, 94, 0.1)",
      };
    case "AtRisk":
      return {
        title: "Some jobs need attention",
        subtitle:
          "One or more backup jobs have errors or warnings. Review the activity log below for details.",
        iconColor: "#f59e0b",
        iconCheck: false,
        borderColor: "rgba(245, 158, 11, 0.15)",
        glowColor: "rgba(245, 158, 11, 0.1)",
      };
    case "Critical":
      return {
        title: "No recent backups",
        subtitle:
          totalJobs === 0
            ? "No backup jobs are configured. Create a backup plan to protect your data."
            : `Backup jobs are configured but no successful backups have been recorded. Check your configuration and storage.`,
        iconColor: "#ef4444",
        iconCheck: false,
        borderColor: "rgba(239, 68, 68, 0.15)",
        glowColor: "rgba(239, 68, 68, 0.1)",
      };
    default:
      return {
        title: "Protection status unknown",
        subtitle: "Unable to determine the current protection state. Check your configuration.",
        iconColor: "#8892a8",
        iconCheck: false,
        borderColor: "rgba(136, 146, 168, 0.15)",
        glowColor: "rgba(136, 146, 168, 0.1)",
      };
  }
}

export default function HeroCard({
  protectionStatus,
  healthStatus,
  totalJobs,
  lastBackupAgo,
  onBackupNow,
  onRestoreFiles,
}: HeroCardProps) {
  const content = getHeroContent(protectionStatus, healthStatus, totalJobs, lastBackupAgo);

  return (
    <div
      style={{
        background: `linear-gradient(135deg, #0f1a2e 0%, #142040 50%, #0d1a30 100%)`,
        border: `1px solid ${content.borderColor}`,
        borderRadius: "16px",
        padding: "32px 36px",
        display: "flex",
        alignItems: "center",
        gap: "32px",
        position: "relative",
        overflow: "hidden",
        transition: theme.transition.normal,
      }}
    >
      {/* Background glow */}
      <div
        style={{
          position: "absolute",
          top: "-50%",
          right: "-10%",
          width: "300px",
          height: "300px",
          background: `radial-gradient(circle, ${content.glowColor} 0%, transparent 70%)`,
          pointerEvents: "none",
        }}
      />

      {/* Icon */}
      <div
        style={{
          width: "72px",
          height: "72px",
          borderRadius: "50%",
          background: `linear-gradient(135deg, ${content.iconColor}33, ${content.iconColor}15)`,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontSize: "34px",
          flexShrink: 0,
          boxShadow: `0 0 30px ${content.glowColor}`,
        }}
      >
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke={content.iconColor} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 2L3 7v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-9-5z" />
          {content.iconCheck && (
            <path d="M9 12l2 2 4-4" stroke={content.iconColor} />
          )}
        </svg>
      </div>

      {/* Text */}
      <div style={{ flex: 1 }}>
        <h2
          style={{
            fontSize: "22px",
            fontWeight: 700,
            color: theme.colors.textPrimary,
            margin: "0 0 6px 0",
          }}
        >
          {content.title}
        </h2>
        <p
          style={{
            fontSize: "14px",
            color: theme.colors.textSecondary,
            margin: "0 0 20px 0",
            lineHeight: 1.5,
          }}
        >
          {content.subtitle}
        </p>
        <div style={{ display: "flex", gap: "12px" }}>
          <Button
            variant="primary"
            size="md"
            icon={<span>{">"}</span>}
            onClick={onBackupNow}
          >
            Run Backup Now
          </Button>
          <Button
            variant="secondary"
            size="md"
            icon={<span>{"<"}</span>}
            onClick={onRestoreFiles}
          >
            Restore Files
          </Button>
        </div>
      </div>

      {/* Decorative server rack illustration */}
      <div
        style={{
          width: "120px",
          height: "120px",
          flexShrink: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          opacity: 0.3,
        }}
      >
        <svg width="100" height="100" viewBox="0 0 100 100" fill="none">
          <rect x="10" y="30" width="80" height="55" rx="4" stroke={content.iconColor} strokeWidth="1.5" fill="none" />
          <rect x="20" y="50" width="60" height="6" rx="2" fill={content.iconColor} opacity="0.3" />
          <rect x="20" y="62" width="40" height="6" rx="2" fill={content.iconColor} opacity="0.2" />
          <rect x="20" y="74" width="50" height="6" rx="2" fill={content.iconColor} opacity="0.15" />
          <circle cx="50" cy="18" r="12" stroke={content.iconColor} strokeWidth="1.5" fill="none" />
          {content.iconCheck && (
            <path d="M44 19 L48 23 L56 15" stroke="#22c55e" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" fill="none" />
          )}
        </svg>
      </div>
    </div>
  );
}
