import { useCallback, useEffect, useState } from "react";
import { api, type Module } from "./api/client";
import { theme } from "./theme";

interface Props {
  module: Module;
  onModuleUpdated: (m: Module) => void;
}

export function ModuleSettings({ module, onModuleUpdated }: Props) {
  const [prefix, setPrefix] = useState(module.prefix);
  const [separator, setSeparator] = useState(module.separator);
  const [digits, setDigits] = useState(module.digits);
  const [defaultClassification, setDefaultClassification] = useState(
    module.default_classification,
  );
  const [requiredAttrs, setRequiredAttrs] = useState<string[]>(
    module.required_attributes ?? [],
  );
  const [newAttr, setNewAttr] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  // Reset form when a different module is selected (not on save)
  const [prevModuleId, setPrevModuleId] = useState(module.id);
  useEffect(() => {
    if (module.id !== prevModuleId) {
      setPrevModuleId(module.id);
      setPrefix(module.prefix);
      setSeparator(module.separator);
      setDigits(module.digits);
      setDefaultClassification(module.default_classification);
      setRequiredAttrs(module.required_attributes ?? []);
      setError(null);
      setSaved(false);
    }
  }, [module, prevModuleId]);

  const handleSave = useCallback(async () => {
    try {
      const updated = await api.updateModule(module.id, {
        prefix,
        separator,
        digits,
        default_classification: defaultClassification,
        required_attributes: requiredAttrs,
      });
      setSaved(true);
      setError(null);
      onModuleUpdated(updated);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save settings");
    }
  }, [module.id, prefix, separator, digits, defaultClassification, requiredAttrs, onModuleUpdated]);

  const handleAddAttr = () => {
    const name = newAttr.trim();
    if (name && !requiredAttrs.includes(name)) {
      setRequiredAttrs([...requiredAttrs, name]);
      setNewAttr("");
    }
  };

  const handleRemoveAttr = (name: string) => {
    setRequiredAttrs(requiredAttrs.filter((a) => a !== name));
  };

  const labelStyle: React.CSSProperties = {
    display: "block",
    marginBottom: theme.spacing.xs,
    fontSize: "0.85rem",
    fontWeight: 600,
    color: theme.colors.textSecondary,
  };

  const inputStyle: React.CSSProperties = {
    padding: theme.spacing.sm,
    width: "100%",
    boxSizing: "border-box",
  };

  return (
    <div style={{ maxWidth: 480 }}>
      {error && (
        <div style={{ color: theme.colors.error, marginBottom: theme.spacing.md }}>
          {error}
        </div>
      )}

      <div style={{ marginBottom: theme.spacing.md }}>
        <label style={labelStyle}>Prefix</label>
        <input
          type="text"
          value={prefix}
          onChange={(e) => setPrefix(e.target.value)}
          placeholder="e.g. REQ"
          style={inputStyle}
        />
      </div>

      <div style={{ marginBottom: theme.spacing.md }}>
        <label style={labelStyle}>Separator</label>
        <input
          type="text"
          value={separator}
          onChange={(e) => setSeparator(e.target.value)}
          placeholder="e.g. -"
          style={inputStyle}
        />
      </div>

      <div style={{ marginBottom: theme.spacing.md }}>
        <label style={labelStyle}>Digits</label>
        <input
          type="number"
          value={digits}
          onChange={(e) => setDigits(Number(e.target.value))}
          min={1}
          max={10}
          style={inputStyle}
        />
      </div>

      <div style={{ marginBottom: theme.spacing.md }}>
        <label style={labelStyle}>Default Classification</label>
        <select
          value={defaultClassification}
          onChange={(e) => setDefaultClassification(e.target.value)}
          style={inputStyle}
        >
          <option value="normative">normative</option>
          <option value="informative">informative</option>
          <option value="heading">heading</option>
        </select>
      </div>

      <div style={{ marginBottom: theme.spacing.md }}>
        <label style={labelStyle}>Required Attributes</label>
        {requiredAttrs.map((attr) => (
          <div
            key={attr}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "4px",
              padding: "2px 8px",
              margin: "2px 4px 2px 0",
              background: theme.colors.bgCode,
              borderRadius: theme.borderRadius,
              fontSize: "0.85rem",
            }}
          >
            {attr}
            <span
              style={{ cursor: "pointer", color: theme.colors.danger, fontWeight: 700 }}
              onClick={() => handleRemoveAttr(attr)}
            >
              x
            </span>
          </div>
        ))}
        <div style={{ display: "flex", gap: theme.spacing.sm, marginTop: theme.spacing.sm }}>
          <input
            type="text"
            value={newAttr}
            onChange={(e) => setNewAttr(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                handleAddAttr();
              }
            }}
            placeholder="Attribute name"
            style={{ padding: theme.spacing.sm, flex: 1 }}
          />
          <button
            onClick={handleAddAttr}
            style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
          >
            Add
          </button>
        </div>
      </div>

      <button
        onClick={handleSave}
        style={{
          padding: `${theme.spacing.sm} ${theme.spacing.lg}`,
          background: theme.colors.primary,
          color: "#fff",
          border: "none",
          borderRadius: theme.borderRadius,
          cursor: "pointer",
        }}
      >
        Save Settings
      </button>
      {saved && (
        <span style={{ marginLeft: theme.spacing.sm, color: theme.colors.success, fontSize: "0.9rem" }}>
          Saved
        </span>
      )}
    </div>
  );
}
