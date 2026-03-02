import { useCallback, useEffect, useState } from "react";
import { api } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  objectId: string;
  onClose: () => void;
  onSaved: () => void;
}

interface Reference {
  type: string;
  path: string;
  description: string;
}

export function ReferencesPanel({ moduleId, objectId, onClose, onSaved }: Props) {
  const [references, setReferences] = useState<Reference[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const fetchObject = useCallback(async () => {
    try {
      const data = await api.getObject(moduleId, objectId);
      const refs = Array.isArray(data.references_) ? data.references_ as Reference[] : [];
      setReferences(refs);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load object");
    }
  }, [moduleId, objectId]);

  useEffect(() => {
    fetchObject();
  }, [fetchObject]);

  const handleAdd = () => {
    setReferences((prev) => [...prev, { type: "url", path: "", description: "" }]);
  };

  const handleUpdate = (index: number, field: keyof Reference, value: string) => {
    setReferences((prev) => {
      const next = [...prev];
      next[index] = { ...next[index], [field]: value };
      return next;
    });
  };

  const handleRemove = (index: number) => {
    setReferences((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await api.updateObject(moduleId, objectId, {
        references: references,
      });
      setError(null);
      onSaved();
      await fetchObject();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save references");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: theme.colors.overlayBg,
        zIndex: 1000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        style={{
          background: theme.colors.bg,
          borderRadius: 8,
          padding: theme.spacing.lg,
          maxWidth: 700,
          width: "90%",
          maxHeight: "80vh",
          overflow: "auto",
          boxShadow: "0 8px 32px rgba(0,0,0,0.2)",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: theme.spacing.sm,
          }}
        >
          <h3 style={{ margin: 0 }}>References for {objectId.slice(0, 8)}...</h3>
          <div style={{ display: "flex", gap: theme.spacing.sm }}>
            <button onClick={handleSave} disabled={saving}>
              {saving ? "Saving..." : "Save"}
            </button>
            <button onClick={onClose}>Close</button>
          </div>
        </div>

        {error && (
          <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>{error}</div>
        )}

        {references.map((ref, i) => (
          <div
            key={i}
            style={{
              display: "flex",
              gap: theme.spacing.sm,
              alignItems: "center",
              marginBottom: theme.spacing.sm,
              padding: theme.spacing.sm,
              background: theme.colors.bgCode,
              borderRadius: theme.borderRadius,
            }}
          >
            <select
              value={ref.type}
              onChange={(e) => handleUpdate(i, "type", e.target.value)}
              style={{ padding: "4px" }}
            >
              <option value="url">URL</option>
              <option value="file">File</option>
              <option value="document">Document</option>
              <option value="other">Other</option>
            </select>
            <input
              type="text"
              value={ref.path}
              onChange={(e) => handleUpdate(i, "path", e.target.value)}
              placeholder="Path or URL"
              style={{ flex: 1, padding: "4px", boxSizing: "border-box" }}
            />
            <input
              type="text"
              value={ref.description}
              onChange={(e) => handleUpdate(i, "description", e.target.value)}
              placeholder="Description"
              style={{ flex: 1, padding: "4px", boxSizing: "border-box" }}
            />
            <button
              onClick={() => handleRemove(i)}
              style={{ padding: "4px 8px", fontSize: "0.8rem" }}
            >
              Remove
            </button>
          </div>
        ))}

        <button
          onClick={handleAdd}
          style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
        >
          Add Reference
        </button>

        {references.length === 0 && (
          <div style={{ padding: theme.spacing.md, textAlign: "center", color: theme.colors.textMuted }}>
            No references yet. Click "Add Reference" to add one.
          </div>
        )}
      </div>
    </div>
  );
}
