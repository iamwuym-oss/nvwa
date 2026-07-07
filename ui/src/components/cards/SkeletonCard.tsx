// ============================================================================
// SkeletonCard.tsx -- Animated loading placeholder matching Dashboard cards
//
// Uses the @keyframes skeletonPulse animation defined in styles.css.
// ============================================================================

import { theme } from "../../theme";

interface SkeletonCardProps {
  height?: number;
  width?: string;
}

export default function SkeletonCard({ height = 80, width = "100%" }: SkeletonCardProps) {
  return (
    <div
      style={{
        width,
        height: `${height}px`,
        animation: "skeletonPulse 1.6s ease-in-out infinite",
        background: `linear-gradient(90deg, ${theme.colors.panel} 0%, ${theme.colors.panelHover} 50%, ${theme.colors.panel} 100%)`,
        backgroundSize: "200% 100%",
        borderRadius: theme.radius.md,
      }}
    />
  );
}
