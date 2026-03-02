import { useCallback, useEffect, useState } from "react";
import { api, type ObjectHistory as ObjHistory } from "./api/client";
import { InlineDiff, AttributesDiff } from "./DiffView";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  objectId: string;
  onClose: () => void;
}

export function ReviewDiffPanel({ moduleId, objectId, onClose }: Props) {
  const [history, setHistory] = useState<ObjHistory[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchHistory = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.listObjectHistory(moduleId, objectId);
      setHistory(data.items.sort((a, b) => a.version - b.version));
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load history");
    } finally {
      setLoading(false);
    }
  }, [moduleId, objectId]);

  useEffect(() => {
    fetchHistory();
  }, [fetchHistory]);

  // Find the last reviewed version (closest version that was reviewed)
  // We compare with the current (latest) version
  const current = history.length > 0 ? history[history.length - 1] : null;
  const previous = history.length > 1 ? history[history.length - 2] : null;

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
          maxWidth: 800,
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
          <h3 style={{ margin: 0 }}>Review Diff for {objectId.slice(0, 8)}...</h3>
          <button onClick={onClose}>Close</button>
        </div>

        {error && (
          <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>{error}</div>
        )}

        {loading && <div style={{ padding: theme.spacing.md, color: theme.colors.textMuted }}>Loading...</div>}

        {!loading && current && previous && (
          <div>
            <div style={{ fontSize: "0.85rem", color: theme.colors.textMuted, marginBottom: theme.spacing.md }}>
              Comparing v{previous.version} (previous) vs v{current.version} (current)
            </div>
            <InlineDiff label="Heading" textA={previous.heading} textB={current.heading} />
            <InlineDiff label="Body" textA={previous.body} textB={current.body} />
            <AttributesDiff attrsA={previous.attribute_values} attrsB={current.attribute_values} />
            {previous.heading === current.heading &&
              previous.body === current.body &&
              JSON.stringify(previous.attribute_values) === JSON.stringify(current.attribute_values) && (
                <p style={{ color: theme.colors.textMuted, fontSize: "0.9rem" }}>
                  No content differences found between versions.
                </p>
              )}
          </div>
        )}

        {!loading && history.length <= 1 && (
          <div style={{ padding: theme.spacing.md, color: theme.colors.textMuted, textAlign: "center" }}>
            No previous version to compare against.
          </div>
        )}
      </div>
    </div>
  );
}
