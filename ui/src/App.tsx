import { useState } from "react";
import MainLayout from "./components/layout/MainLayout";
import Dashboard from "./pages/Dashboard/Dashboard";
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
      case "dashboard": return <Dashboard onNavigate={setCurrentPage} />;
      case "backup":    return <Backup />;
      case "restore":   return <Restore />;
      case "history":   return <History />;
      case "schedule":  return <Schedule />;
      case "settings":  return <Settings />;
      case "clone":     return <Clone />;
    }
  };

  return (
    <MainLayout currentPage={currentPage} onNavigate={setCurrentPage}>
      {renderPage()}
    </MainLayout>
  );
}

export default App;
