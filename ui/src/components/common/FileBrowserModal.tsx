// ============================================================================
// FileBrowserModal.tsx -- In-app directory browser modal
//
// Replaces OS-native directory dialog (tauri-plugin-dialog) with Nüwa's own
// UI for browsing local Roots and directories.  Users can navigate the
// filesystem tree inside the app and select a folder.
// ============================================================================

import { useEffect, useState, useCallback } from "react";
import { listRoots, listDirectory, RootEntry, FsEntry } from "../../api/fileBrowserApi";
import Button from "./Button";
import { theme } from "../../theme";

interface FileBrowserModalProps {
  open: boolean;
  onSelect: (path: string) => void;
  onCancel: () => void;
  title?: string;
}

const overlayStyle: React.CSSProperties = {
  position: "fixed", top: 0, left: 0, right: 0, bottom: 0,
  background: theme.colors.overlay, display: "flex", alignItems: "center",
  justifyContent: "center", zIndex: 1000,
};

const modalStyle: React.CSSProperties = {
  width: "680px", height: "500px", background: theme.colors.panelElevated,
  border: "1px solid " + theme.colors.panelBorder, borderRadius: theme.radius.lg,
  display: "flex", flexDirection: "column", overflow: "hidden", boxShadow: theme.shadow.elevated,
};

const headerStyle: React.CSSProperties = {
  padding: "16px 20px", borderBottom: "1px solid " + theme.colors.divider,
  fontSize: theme.font.sizeLg, fontWeight: 600, color: theme.colors.textPrimary,
};

const bodyStyle: React.CSSProperties = { flex: 1, display: "flex", overflow: "hidden" };

const sidebarStyle: React.CSSProperties = {
  width: "200px", borderRight: "1px solid " + theme.colors.divider,
  overflowY: "auto", padding: "8px 0",
};

const contentStyle: React.CSSProperties = { flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" };

const pathBarStyle: React.CSSProperties = {
  padding: "10px 16px", borderBottom: "1px solid " + theme.colors.divider,
  fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, fontFamily: theme.font.mono,
  whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
};

const listStyle: React.CSSProperties = { flex: 1, overflowY: "auto", padding: "4px 0" };

const itemBase: React.CSSProperties = {
  display: "flex", alignItems: "center", gap: "8px", padding: "7px 16px",
  cursor: "pointer", fontSize: theme.font.sizeMd, color: theme.colors.textPrimary,
  border: "none", background: "transparent", width: "100%", textAlign: "left",
  boxSizing: "border-box", fontFamily: theme.font.family,
};

const itemActive: React.CSSProperties = {
  ...itemBase, background: theme.colors.primaryDim, color: theme.colors.primary,
};

const footerStyle: React.CSSProperties = {
  padding: "12px 20px", borderTop: "1px solid " + theme.colors.divider,
  display: "flex", justifyContent: "flex-end", gap: "8px",
};

const selectedPathStyle: React.CSSProperties = {
  padding: "8px 16px", borderTop: "1px solid " + theme.colors.divider,
  fontSize: theme.font.sizeSm, color: theme.colors.textSecondary, fontFamily: theme.font.mono,
  whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
};

const emptyStyle: React.CSSProperties = {
  padding: "32px 16px", textAlign: "center", color: theme.colors.textMuted, fontSize: theme.font.sizeSm,
};

export default function FileBrowserModal({ open, onSelect, onCancel, title }: FileBrowserModalProps) {
  const [Roots, setRoots] = useState<RootEntry[]>([]);
  const [currentPath, setCurrentPath] = useState<string>("");
  const [entries, setEntries] = useState<FsEntry[]>([]);
  const [selectedPath, setSelectedPath] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    listRoots().then((d) => {
      setRoots(d);
      if (d.length > 0) navigateTo(d[0].path);
    }).catch(() => {});
  }, [open]);

  const navigateTo = useCallback(async (path: string) => {
    setLoading(true); setError(null);
    setCurrentPath(path); setSelectedPath(path);
    try {
      setEntries(await listDirectory(path));
    } catch (err: any) {
      setError(err?.message || "Cannot read directory");
      setEntries([]);
    } finally {
      setLoading(false);
    }
  }, []);

  const goUp = useCallback(() => {
    if (!currentPath) return;
    const parts = currentPath.replace(/\\/g, "/").replace(/\/$/, "").split("/");
    if (parts.length <= 1) return;
    parts.pop();
    const parent = parts.join("\\");
    navigateTo(parent.match(/^[A-Za-z]:$/i) ? parent + "\\" : parent);
  }, [currentPath, navigateTo]);

  const isRoot = !!currentPath.match(/^[A-Za-z]:\\$/);

  if (!open) return null;

  return (
    <div style={overlayStyle} onClick={onCancel}>
      <div style={modalStyle} onClick={(e) => e.stopPropagation()}>
        <div style={headerStyle}>{title || "Select Folder"}</div>
        <div style={bodyStyle}>
          <div style={sidebarStyle}>
            {Roots.map((drive) => (
              <button key={drive.path} style={currentPath === drive.path ? itemActive : itemBase} onClick={() => navigateTo(drive.path)}>
                <span>💾</span>
                <span>{drive.label}</span>
              </button>
            ))}
          </div>
          <div style={contentStyle}>
            <div style={pathBarStyle}>
              {!isRoot && <span onClick={goUp} style={{ cursor: "pointer", color: theme.colors.primary, marginRight: "8px" }}>↑ ..</span>}
              {currentPath || "Select a drive"}
            </div>
            <div style={listStyle}>
              {loading && <div style={emptyStyle}>Loading...</div>}
              {!loading && error && <div style={{...emptyStyle, color: theme.colors.error}}>{error}</div>}
              {!loading && !error && entries.length === 0 && <div style={emptyStyle}>Empty folder</div>}
              {!loading && !error && entries.map((entry) => (
                <button key={entry.path} style={selectedPath === entry.path ? itemActive : itemBase} onClick={() => navigateTo(entry.path)}>
                  <span>📁</span>
                  <span>{entry.name}</span>
                </button>
              ))}
            </div>
            <div style={selectedPathStyle}>{selectedPath || "No folder selected"}</div>
          </div>
        </div>
        <div style={footerStyle}>
          <Button variant="secondary" onClick={onCancel}>Cancel</Button>
          <Button variant="primary" onClick={() => { if (selectedPath) onSelect(selectedPath); }} disabled={!selectedPath}>
            Select This Folder
          </Button>
        </div>
      </div>
    </div>
  );
}
