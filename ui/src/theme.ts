// Nuwa Backup Desktop GUI — Theme System
// Premium Dark Enterprise UI color palette

export const theme = {
  colors: {
    // Backgrounds
    background:       "#070B16",
    panel:            "#101827",
    panelHover:       "#1a2335",
    panelBorder:      "#1e2a45",
    panelElevated:    "#141e33",

    // Primary / Accent
    primary:          "#1683ff",
    primaryHover:     "#3a9aff",
    primaryDim:       "rgba(22, 131, 255, 0.12)",
    primaryGlow:      "rgba(22, 131, 255, 0.25)",

    // Text
    textPrimary:      "#e8edf5",
    textSecondary:    "#8892a8",
    textMuted:        "#5a6278",

    // Status
    success:          "#22c55e",
    successDim:       "rgba(34, 197, 94, 0.12)",
    warning:          "#f59e0b",
    warningDim:       "rgba(245, 158, 11, 0.12)",
    error:            "#ef4444",
    errorDim:         "rgba(239, 68, 68, 0.12)",

    // Sidebar
    sidebarBg:        "#0a0f1e",
    sidebarHover:     "#111827",
    sidebarActive:    "rgba(22, 131, 255, 0.15)",
    sidebarActiveBorder: "#1683ff",
    sidebarText:      "#5a6278",
    sidebarTextActive:"#e8edf5",

    // Misc
    divider:          "rgba(255,255,255,0.06)",
    overlay:          "rgba(0,0,0,0.5)",
    scrollbar:        "#1e2a45",
    scrollbarHover:   "#2a3a5a",
  },

  spacing: {
    xs: "4px",
    sm: "8px",
    md: "16px",
    lg: "24px",
    xl: "32px",
    xxl: "48px",
  },

  radius: {
    sm: "4px",
    md: "8px",
    lg: "12px",
    xl: "16px",
    full: "9999px",
  },

  font: {
    family: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial, sans-serif',
    mono: '"SF Mono", "Cascadia Code", Consolas, monospace',
    sizeXs: "11px",
    sizeSm: "12px",
    sizeMd: "14px",
    sizeLg: "16px",
    sizeXl: "20px",
    sizeXxl: "24px",
    sizeHero: "32px",
  },

  shadow: {
    card: "0 1px 3px rgba(0,0,0,0.3), 0 1px 2px rgba(0,0,0,0.2)",
    elevated: "0 4px 12px rgba(0,0,0,0.4), 0 2px 4px rgba(0,0,0,0.3)",
    glow: "0 0 20px rgba(22, 131, 255, 0.15)",
  },

  transition: {
    fast: "0.15s ease",
    normal: "0.2s ease",
    slow: "0.3s ease",
  },

  sidebarWidth: "240px",
  topbarHeight: "64px",
};

export type Theme = typeof theme;
export default theme;
