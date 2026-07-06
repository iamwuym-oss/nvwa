import { Page } from "../App";

interface SidebarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

const navItems: { page: Page; label: string; icon: string }[] = [
  { page: "dashboard", label: "Dashboard",    icon: "📊" },
  { page: "backup",    label: "Backup",       icon: "⬆" },
  { page: "restore",   label: "Restore",      icon: "⬇" },
  { page: "history",   label: "History",      icon: "📋" },
  { page: "schedule",  label: "Schedule",     icon: "⏰" },
  { page: "settings",  label: "Settings",     icon: "⚙" },
  { page: "clone",     label: "Clone",        icon: "📀" },
];

function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  const cls = (p: string) => "sidebar-item" + (currentPage === p ? " active" : "");
  return (
    <nav className="sidebar">
      <div className="sidebar-header">
        <span className="sidebar-logo">🛡</span>
        <span className="sidebar-title">Nuwa Backup</span>
      </div>
      <ul className="sidebar-nav">
        {navItems.map(({ page, label, icon }) => (
          <li key={page}>
            <button className={cls(page)} onClick={() => onNavigate(page)}>
              <span className="sidebar-icon">{icon}</span>
              <span className="sidebar-label">{label}</span>
              {page === "clone" && <span className="sidebar-badge">Soon</span>}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}

export default Sidebar;
