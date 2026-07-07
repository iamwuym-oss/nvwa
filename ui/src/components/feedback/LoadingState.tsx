// ============================================================================
// LoadingState.tsx -- Skeleton loading layout matching the Dashboard grid
//
// Renders an animated placeholder that mirrors the actual Dashboard card layout
// so the page does not "jump" when real data arrives.
// ============================================================================

import { theme } from "../../theme";
import SkeletonCard from "../cards/SkeletonCard";

export default function LoadingState() {
  return (
    <div style={{ maxWidth: "1200px", margin: "0 auto", opacity: 0.7 }}>
      {/* Hero Card skeleton */}
      <div
        style={{
          marginBottom: "20px",
          height: "160px",
          background: `linear-gradient(135deg, ${theme.colors.panel} 0%, ${theme.colors.panelHover} 50%, ${theme.colors.panel} 100%)`,
          borderRadius: theme.radius.xl,
          border: `1px solid ${theme.colors.panelBorder}`,
          animation: "skeletonPulse 1.6s ease-in-out infinite",
          backgroundSize: "200% 100%",
        }}
      />

      {/* Metric cards row */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(4, 1fr)",
          gap: "16px",
          marginBottom: "20px",
        }}
      >
        {[0, 1, 2, 3].map((i) => (
          <SkeletonCard key={i} height={120} />
        ))}
      </div>

      {/* Chart + Activity row */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "2fr 1fr",
          gap: "16px",
          marginBottom: "20px",
        }}
      >
        <SkeletonCard height={220} />
        <SkeletonCard height={220} />
      </div>

      {/* Bottom panels row */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: "16px",
        }}
      >
        {[0, 1, 2].map((i) => (
          <SkeletonCard key={i} height={180} />
        ))}
      </div>
    </div>
  );
}
