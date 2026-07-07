import { Page } from "../../App";
import { theme } from "../../theme";

interface SidebarProps { currentPage: Page; onNavigate: (page: Page) => void; }

const navItems: { page: Page; label: string; icon: string }[] = [
  { page: "dashboard", label: "Dashboard", icon: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" },
  { page: "backup",    label: "Backup",    icon: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" },
  { page: "restore",   label: "Restore",   icon: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" },
  { page: "history",   label: "History",   icon: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" },
  { page: "schedule",  label: "Schedule",  icon: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" },
  { page: "settings",  label: "Settings",  icon: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065zM15 12a3 3 0 11-6 0 3 3 0 016 0z" },
  { page: "clone",     label: "Clone",     icon: "M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" },
];

export default function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <nav style={{
      width: theme.sidebarWidth, minWidth: theme.sidebarWidth,
      background: theme.colors.sidebarBg,
      display: "flex", flexDirection: "column",
      userSelect: "none", borderRight: "1px solid " + theme.colors.divider,
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: "10px", padding: "20px 18px", borderBottom: "1px solid " + theme.colors.divider }}>
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke={theme.colors.primary} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 2L3 7v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-9-5z"/>
        </svg>
        <span style={{ fontSize: "16px", fontWeight: 700, color: "#ffffff", letterSpacing: "0.3px" }}>Nuwa Backup</span>
      </div>
      <ul style={{ listStyle: "none", padding: "8px 0", flex: 1, margin: 0 }}>
        {navItems.map(({ page, label, icon }) => {
          const active = currentPage === page;
          return (
            <li key={page} style={{ padding: "0 8px" }}>
              <button onClick={() => onNavigate(page)} style={{
                display: "flex", alignItems: "center", gap: "10px", width: "100%",
                padding: "10px 12px", background: active ? theme.colors.sidebarActive : "transparent",
                border: "none", borderRadius: theme.radius.md,
                color: active ? theme.colors.sidebarTextActive : theme.colors.sidebarText,
                fontSize: "14px", fontWeight: active ? 600 : 400,
                cursor: "pointer", textAlign: "left", position: "relative",
                transition: theme.transition.fast,
              }}>
                {active && <div style={{ position: "absolute", left: 0, top: "4px", bottom: "4px", width: "3px", borderRadius: "0 2px 2px 0", background: theme.colors.sidebarActiveBorder, boxShadow: "0 0 8px rgba(22,131,255,0.5)" }} />}
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ flexShrink: 0 }}><path d={icon}/></svg>
                <span>{label}</span>
                {page === "clone" && <span style={{ fontSize: "10px", background: "rgba(245,158,11,0.15)", color: theme.colors.warning, padding: "1px 6px", borderRadius: theme.radius.full, fontWeight: 600, marginLeft: "auto" }}>Soon</span>}
              </button>
            </li>
          );
        })}
      </ul>
      <div style={{ padding: "16px", borderTop: "1px solid " + theme.colors.divider, display: "flex", alignItems: "center", gap: "10px" }}>
        <div style={{ width: "8px", height: "8px", borderRadius: "50%", background: theme.colors.success, boxShadow: "0 0 8px " + theme.colors.success, flexShrink: 0 }} />
        <div>
          <div style={{ fontSize: "12px", fontWeight: 600, color: theme.colors.sidebarTextActive }}>Protection Enabled</div>
          <div style={{ fontSize: "11px", color: theme.colors.sidebarText }}>Your data is safe</div>
        </div>
      </div>
    </nav>
  );
}
