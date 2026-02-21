import { useEffect, useState } from "react";
import Markdown from "react-markdown";
import { api, type ObjectHistory as ObjectHistoryModel } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  objectId: string;
  onClose: () => void;
}

export function ObjectHistory({ moduleId, objectId, onClose }: Props) {
  const [history, setHistory] = useState<ObjectHistoryModel[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .listObjectHistory(moduleId, objectId)
      .then((data) => setHistory(data.items))
      .catch((err) =>
        setError(err instanceof Error ? err.message : "Failed to load history"),
      );
  }, [moduleId, objectId]);

  const formatAttrs = (attrs: Record<string, unknown> | null): string => {
    if (!attrs || Object.keys(attrs).length === 0) return "\u2014";
    return Object.entries(attrs)
      .map(([k, v]) => `${k}: ${v}`)
      .join(", ");
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
          maxWidth: 900,
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
          <h3 style={{ margin: 0 }}>History for {objectId.slice(0, 8)}...</h3>
          <button onClick={onClose}>Close</button>
        </div>

        {error && <div style={{ color: theme.colors.error }}>{error}</div>}

        <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.9rem" }}>
          <thead>
            <tr>
              <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "0.25rem" }}>
                Version
              </th>
              <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "0.25rem" }}>
                Type
              </th>
              <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "0.25rem" }}>
                Heading
              </th>
              <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "0.25rem" }}>
                Body
              </th>
              <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "0.25rem" }}>
                Attributes
              </th>
              <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "0.25rem" }}>
                Changed At
              </th>
            </tr>
          </thead>
          <tbody>
            {history.map((h) => (
              <tr key={h.id}>
                <td style={{ padding: "0.25rem", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                  {h.version}
                </td>
                <td style={{ padding: "0.25rem", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                  {h.change_type}
                </td>
                <td style={{ padding: "0.25rem", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                  {h.heading ?? "\u2014"}
                </td>
                <td style={{ padding: "0.25rem", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                  {h.body ? <Markdown>{h.body}</Markdown> : "\u2014"}
                </td>
                <td style={{ padding: "0.25rem", borderBottom: `1px solid ${theme.colors.borderLight}`, fontSize: "0.85rem" }}>
                  {formatAttrs(h.attribute_values)}
                </td>
                <td style={{ padding: "0.25rem", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                  {new Date(h.changed_at).toLocaleString()}
                </td>
              </tr>
            ))}
            {history.length === 0 && (
              <tr>
                <td colSpan={6} style={{ padding: theme.spacing.sm, textAlign: "center", color: theme.colors.textMuted }}>
                  No history records.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
