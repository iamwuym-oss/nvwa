import { Page } from "../../App";
import Sidebar from "./Sidebar";
import TopBar from "./TopBar";
import { theme } from "../../theme";

interface MainLayoutProps { currentPage: Page; onNavigate: (page: Page) => void; children: React.ReactNode; }

export default function MainLayout({ currentPage, onNavigate, children }: MainLayoutProps) {
  return (
    <div style={{ display: "flex", height: "100vh", width: "100vw", background: theme.colors.background, color: theme.colors.textPrimary, fontFamily: theme.font.family, overflow: "hidden" }}>
      <Sidebar currentPage={currentPage} onNavigate={onNavigate} />
      <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        <TopBar currentPage={currentPage} />
        <main style={{ flex: 1, overflowY: "auto", overflowX: "hidden", padding: "24px", background: theme.colors.background }}>
          {children}
        </main>
      </div>
    </div>
  );
}
