import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import Button from "./Button";
import { theme } from "../../theme";

interface PathInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  label?: string;
  disabled?: boolean;
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

export default function PathInput({ value, onChange, placeholder, disabled }: PathInputProps) {
  const [browsing, setBrowsing] = useState(false);

  const handleBrowse = async () => {
    setBrowsing(true);
    try {
      const selected = await open({ directory: true, multiple: false, title: "Select Folder" });
      if (selected !== null) {
        onChange(selected);
      }
    } catch {
      // User cancelled or backend error -- keep original value
    } finally {
      setBrowsing(false);
    }
  };

  return (
    <div style={rowStyle}>
      <input
        style={inputBase}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        disabled={disabled}
      />
      <Button variant="secondary" size="md" onClick={handleBrowse} disabled={disabled || browsing} style={browseBtnStyle}>
        {browsing ? "..." : "Browse"}
      </Button>
    </div>
  );
}
