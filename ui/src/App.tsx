import { useState } from "react";
import Sidebar from "./components/Sidebar";
import TopBar from "./components/TopBar";
import Dashboard from "./pages/Dashboard";
import Backup from "./pages/Backup";
import Restore from "./pages/Restore";
import History from "./pages/History";
import Schedule from "./pages/Schedule";
import Settings from "./pages/Settings";
import Clone from "./pages/Clone";

export type Page =
  | "dashboard"
  | "backup"
  | "restore"
  | "history"
  | "schedule"
  | "settings"
  | "clone";

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("dashboard");

  const renderPage = () => {
    switch (currentPage) {
      case "dashboard": return <Dashboard />;
      case "backup":    return <Backup />;
      case "restore":   return <Restore />;
      case "history":   return <History />;
      case "schedule":  return <Schedule />;
      case "settings":  return <Settings />;
      case "clone":     return <Clone />;
    }
  };

  return (
    <div className="app-container">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <div className="main-area">
        <TopBar currentPage={currentPage} />
        <div className="content-area">
          {renderPage()}
        </div>
      </div>
    </div>
  );
}

export default App;
