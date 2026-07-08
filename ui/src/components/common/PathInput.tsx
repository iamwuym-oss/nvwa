// ============================================================================
// PathInput.tsx -- Text input with in-app Browse button
//
// Browse opens Nüwa's own FileBrowserModal (not OS directory dialog).
// Users can still manually type paths.
// ============================================================================

import { useState } from "react";
import Button from "./Button";
import FileBrowserModal from "./FileBrowserModal";
import { theme } from "../../theme";

interface PathInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  label?: string;
  disabled?: boolean;
  browseTitle?: string;
}

const inputBase: React.CSSProperties = {
  flex: 1,
  padding: "10px 12px",
  background: theme.colors.background,
  border: "1px solid " + theme.colors.panelBorder,
  borderRight: "none",
  borderRadius: theme.radius.md + " 0 0 " + theme.radius.md,
  color: theme.colors.textPrimary,
  fontSize: theme.font.sizeMd,
  fontFamily: theme.font.mono,
  outline: "none",
  boxSizing: "border-box",
};

const rowStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "stretch",
};

const browseBtnStyle: React.CSSProperties = {
  borderRadius: "0 " + theme.radius.md + " " + theme.radius.md + " 0",
  whiteSpace: "nowrap",
};

export default function PathInput({ value, onChange, placeholder, disabled, browseTitle }: PathInputProps) {
  const [browserOpen, setBrowserOpen] = useState(false);

  return (
    <>
      <div style={rowStyle}>
        <input
          style={inputBase}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
        />
        <Button
          variant="secondary"
          size="md"
          onClick={() => setBrowserOpen(true)}
          disabled={disabled}
          style={browseBtnStyle}
        >
          📁 Browse
        </Button>
      </div>

      <FileBrowserModal
        open={browserOpen}
        title={browseTitle || "Select Folder"}
        onSelect={(path) => {
          onChange(path);
          setBrowserOpen(false);
        }}
        onCancel={() => setBrowserOpen(false)}
      />
    </>
  );
}
