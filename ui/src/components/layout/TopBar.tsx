import { Page } from "../../App";
import { theme } from "../../theme";

interface TopBarProps { currentPage: Page; }

const cfg: Record<Page, { title: string; subtitle: string }> = {
  dashboard: { title: "Dashboard", subtitle: "Overview of your backup environment" },
  backup:    { title: "Backup",    subtitle: "Manage your backup jobs" },
  restore:   { title: "Restore",   subtitle: "Restore files from backup points" },
  history:   { title: "History",   subtitle: "View backup history and logs" },
  schedule:  { title: "Schedule",  subtitle: "Configure scheduled backups" },
  settings:  { title: "Settings",  subtitle: "Application configuration" },
  clone:     { title: "Clone",     subtitle: "Disk cloning (Phase 5)" },
};

export default function TopBar({ currentPage }: TopBarProps) {
  const c = cfg[currentPage];
  return (
    <header style={{
      height: theme.topbarHeight, minHeight: theme.topbarHeight,
      display: "flex", alignItems: "center", justifyContent: "space-between",
      padding: "0 24px", background: theme.colors.background,
      borderBottom: "1px solid " + theme.colors.divider,
    }}>
      <div>
        <h1 style={{ fontSize: "18px", fontWeight: 700, color: theme.colors.textPrimary, margin: 0 }}>{c.title}</h1>
        <p style={{ fontSize: "12px", color: theme.colors.textMuted, margin: "2px 0 0 0" }}>{c.subtitle}</p>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
        <div style={{ position: "relative" }}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={theme.colors.textMuted} strokeWidth="2" style={{ position: "absolute", left: "10px", top: "50%", transform: "translateY(-50%)" }}>
            <circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/>
          </svg>
          <input placeholder="Search..." style={{
            background: "rgba(255,255,255,0.04)", border: "1px solid " + theme.colors.divider,
            borderRadius: theme.radius.md, padding: "6px 12px 6px 32px",
            fontSize: "13px", color: theme.colors.textPrimary, width: "200px",
            outline: "none", fontFamily: theme.font.family,
          }} />
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "6px", padding: "4px 10px", background: theme.colors.successDim, borderRadius: theme.radius.full }}>
          <div style={{ width: "6px", height: "6px", borderRadius: "50%", background: theme.colors.success }} />
          <span style={{ fontSize: "11px", fontWeight: 600, color: theme.colors.success }}>All Systems Operational</span>
        </div>
        <button style={{ background: "none", border: "none", cursor: "pointer", color: theme.colors.textMuted, padding: "4px", position: "relative" }}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/></svg>
          <span style={{ position: "absolute", top: "2px", right: "2px", width: "6px", height: "6px", borderRadius: "50%", background: theme.colors.error }} />
        </button>
        <div style={{ width: "32px", height: "32px", borderRadius: "50%", background: theme.colors.primaryDim, display: "flex", alignItems: "center", justifyContent: "center", fontSize: "13px", fontWeight: 700, color: theme.colors.primary, cursor: "pointer" }}>A</div>
      </div>
    </header>
  );
}
