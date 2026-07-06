import { Page } from "../App";

const pageTitles: Record<Page, string> = {
  dashboard: "Dashboard",
  backup:    "Backup",
  restore:   "Restore",
  history:   "History",
  schedule:  "Schedule",
  settings:  "Settings",
  clone:     "Disk Clone",
};

interface TopBarProps {
  currentPage: Page;
}

function TopBar({ currentPage }: TopBarProps) {
  return (
    <header className="topbar">
      <h1 className="topbar-title">{pageTitles[currentPage]}</h1>
      <span className="topbar-version">v0.1.0</span>
    </header>
  );
}

export default TopBar;
