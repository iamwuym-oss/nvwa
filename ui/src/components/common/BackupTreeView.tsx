// ============================================================================
// BackupTreeView.tsx -- Tree view for browsing backup file contents
//
// Converts a flat list of RestoreFileEntry items (from get_restore_preview)
// into an expandable directory tree, similar to Acronis backup content
// explorer or Windows file explorer.
//
// Data flow:
//   get_restore_preview(backupId) -> RestorePreview.files (flat)
//   -> buildTree(flatList) -> FileTreeNode[] (tree)
//   -> BackupTreeView renders tree with expand/collapse
// ============================================================================

import { useState, useMemo } from "react";
import { RestoreFileEntry } from "../../api/restoreApi";
import { theme } from "../../theme";

// ---------------------------------------------------------------------------
// Tree node model
// ---------------------------------------------------------------------------

export interface FileTreeNode {
  name: string;
  path: string;
  isDirectory: boolean;
  size?: number;
  modifiedTime?: string;
  children: FileTreeNode[];
}

// ---------------------------------------------------------------------------
// Build tree from flat file list
// ---------------------------------------------------------------------------

function buildTree(files: RestoreFileEntry[]): FileTreeNode[] {
  const root: FileTreeNode[] = [];
  const map = new Map<string, FileTreeNode>();

  for (const file of files) {
    const segments = file.relative_path.split(/[\\/]/);
    let currentPath = "";

    for (let i = 0; i < segments.length; i++) {
      const seg = segments[i];
      if (!seg) continue;

      const isFile = i === segments.length - 1;
      const parentPath = currentPath;
      currentPath = currentPath ? `${currentPath}\\${seg}` : seg;

      if (map.has(currentPath)) {
        if (isFile) {
          // Update with file metadata
          const node = map.get(currentPath)!;
          node.isDirectory = false;
          node.size = file.size_bytes;
          node.modifiedTime = file.modified_time;
        }
        continue;
      }

      const node: FileTreeNode = {
        name: seg,
        path: currentPath,
        isDirectory: !isFile,
        size: isFile ? file.size_bytes : undefined,
        modifiedTime: isFile ? file.modified_time : undefined,
        children: [],
      };

      map.set(currentPath, node);

      if (parentPath && map.has(parentPath)) {
        map.get(parentPath)!.children.push(node);
      } else {
        root.push(node);
      }
    }
  }

  // Sort: directories first, then alphabetically
  const sortNodes = (nodes: FileTreeNode[]) => {
    nodes.sort((a, b) => {
      if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    for (const n of nodes) sortNodes(n.children);
  };
  sortNodes(root);

  return root;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return val.toFixed(1) + " " + units[i];
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

const treeContainer: React.CSSProperties = {
  maxHeight: "400px",
  overflowY: "auto",
  background: theme.colors.background,
  borderRadius: theme.radius.md,
  border: "1px solid " + theme.colors.panelBorder,
  fontFamily: theme.font.family,
};

const rowBase: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  padding: "3px 8px",
  cursor: "default",
  fontSize: theme.font.sizeSm,
  color: theme.colors.textPrimary,
  whiteSpace: "nowrap",
  gap: "4px",
};

const toggleIcon: React.CSSProperties = {
  width: "16px",
  textAlign: "center",
  flexShrink: 0,
  color: theme.colors.textMuted,
  cursor: "pointer",
  userSelect: "none",
  fontSize: "10px",
};

const fileIcon: React.CSSProperties = {
  width: "16px",
  textAlign: "center",
  flexShrink: 0,
};

const nameStyle: React.CSSProperties = {
  flex: 1,
  overflow: "hidden",
  textOverflow: "ellipsis",
  marginLeft: "4px",
};

const metaStyle: React.CSSProperties = {
  color: theme.colors.textMuted,
  fontSize: theme.font.sizeXs,
  marginLeft: "auto",
  paddingLeft: "16px",
  flexShrink: 0,
};

// ---------------------------------------------------------------------------
// TreeNode component (recursive)
// ---------------------------------------------------------------------------

interface TreeNodeProps {
  node: FileTreeNode;
  depth: number;
  selectedPath: string | null;
  onSelect: (path: string) => void;
}

function TreeNode({ node, depth, selectedPath, onSelect }: TreeNodeProps) {
  const [expanded, setExpanded] = useState(false);
  const hasChildren = node.isDirectory && node.children.length > 0;
  const isSelected = selectedPath === node.path;

  const indent = depth * 20;

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setExpanded(!expanded);
  };

  return (
    <>
      <div
        style={{
          ...rowBase,
          paddingLeft: `${8 + indent}px`,
          background: isSelected ? theme.colors.primaryDim : "transparent",
        }}
        onClick={() => onSelect(node.path)}
      >
        {/* Toggle */}
        <span style={toggleIcon} onClick={handleToggle}>
          {hasChildren ? (expanded ? "▼" : "▶") : ""}
        </span>
        {/* Icon */}
        <span style={fileIcon}>{node.isDirectory ? (expanded ? "📂" : "📁") : "📄"}</span>
        {/* Name */}
        <span style={nameStyle} title={node.path}>{node.name}</span>
        {/* Size / Meta */}
        <span style={metaStyle}>
          {node.isDirectory
            ? `${node.children.length} items`
            : node.size ? formatBytes(node.size) : ""
          }
        </span>
      </div>
      {/* Children */}
      {node.isDirectory && expanded && node.children.map((child) => (
        <TreeNode
          key={child.path}
          node={child}
          depth={depth + 1}
          selectedPath={selectedPath}
          onSelect={onSelect}
        />
      ))}
    </>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

interface BackupTreeViewProps {
  files: RestoreFileEntry[];
  selectedPath: string | null;
  onSelect: (path: string) => void;
}

export default function BackupTreeView({ files, selectedPath, onSelect }: BackupTreeViewProps) {
  const tree = useMemo(() => buildTree(files), [files]);

  if (files.length === 0) {
    return (
      <div style={{ ...treeContainer, padding: "32px 16px", textAlign: "center", color: theme.colors.textMuted }}>
        No files to restore in this backup point.
      </div>
    );
  }

  return (
    <div style={treeContainer}>
      {tree.map((node) => (
        <TreeNode
          key={node.path}
          node={node}
          depth={0}
          selectedPath={selectedPath}
          onSelect={onSelect}
        />
      ))}
    </div>
  );
}
