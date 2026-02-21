import { useState } from "react";
import { api, type ImpactObject, type ImpactResponse } from "./api/client";
import { ImpactGraph } from "./ImpactGraph";
import { theme } from "./theme";

interface Props {
  objectId: string;
  onClose: () => void;
}

export function ImpactPanel({ objectId, onClose }: Props) {
  const [direction, setDirection] = useState("both");
  const [maxDepth, setMaxDepth] = useState(5);
  const [result, setResult] = useState<ImpactResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<"list" | "graph">("list");

  const handleAnalyze = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.getImpact(objectId, direction, maxDepth);
      setResult(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to run impact analysis");
    } finally {
      setLoading(false);
    }
  };

  // Group objects by depth
  const groupedByDepth = new Map<number, ImpactObject[]>();
  if (result) {
    for (const obj of result.objects) {
      const list = groupedByDepth.get(obj.depth) ?? [];
      list.push(obj);
      groupedByDepth.set(obj.depth, list);
    }
  }

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
          <h3 style={{ margin: 0 }}>Impact Analysis for {objectId.slice(0, 8)}...</h3>
          <button onClick={onClose}>Close</button>
        </div>

        {error && <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>{error}</div>}

        <div style={{ display: "flex", gap: theme.spacing.md, alignItems: "center", marginBottom: theme.spacing.md, flexWrap: "wrap" }}>
          <div style={{ display: "flex", gap: theme.spacing.sm, alignItems: "center" }}>
            <label style={{ fontSize: "0.9rem", fontWeight: 600 }}>Direction:</label>
            {(["forward", "backward", "both"] as const).map((d) => (
              <label key={d} style={{ display: "flex", alignItems: "center", gap: "4px", fontSize: "0.9rem" }}>
                <input
                  type="radio"
                  name="direction"
                  value={d}
                  checked={direction === d}
                  onChange={() => setDirection(d)}
                />
                {d.charAt(0).toUpperCase() + d.slice(1)}
              </label>
            ))}
          </div>

          <div style={{ display: "flex", gap: theme.spacing.xs, alignItems: "center" }}>
            <label style={{ fontSize: "0.9rem", fontWeight: 600 }}>Max Depth:</label>
            <input
              type="number"
              min={1}
              max={10}
              value={maxDepth}
              onChange={(e) => setMaxDepth(Number(e.target.value))}
              style={{ width: "60px", padding: "0.3rem" }}
            />
          </div>

          <button
            onClick={handleAnalyze}
            disabled={loading}
            style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
          >
            {loading ? "Analyzing..." : "Analyze"}
          </button>

          {result && (
            <div style={{ display: "flex", gap: 2, marginLeft: theme.spacing.sm }}>
              {(["list", "graph"] as const).map((mode) => (
                <button
                  key={mode}
                  onClick={() => setViewMode(mode)}
                  style={{
                    padding: `${theme.spacing.xs} ${theme.spacing.sm}`,
                    fontSize: "0.85rem",
                    background: viewMode === mode ? theme.colors.tabActive : "transparent",
                    color: viewMode === mode ? "#fff" : theme.colors.text,
                    border: `1px solid ${theme.colors.border}`,
                    borderRadius: theme.borderRadius,
                    cursor: "pointer",
                  }}
                >
                  {mode.charAt(0).toUpperCase() + mode.slice(1)}
                </button>
              ))}
            </div>
          )}
        </div>

        {result && (
          <div>
            <div style={{ fontSize: "0.85rem", color: theme.colors.textMuted, marginBottom: theme.spacing.sm }}>
              Found {result.objects.length} linked object{result.objects.length !== 1 ? "s" : ""} (direction: {result.direction}, max depth: {result.max_depth})
            </div>

            {result.objects.length === 0 ? (
              <div style={{ padding: theme.spacing.md, textAlign: "center", color: theme.colors.textMuted }}>
                No linked objects found.
              </div>
            ) : viewMode === "graph" ? (
              <ImpactGraph response={result} rootId={objectId} />
            ) : (
              Array.from(groupedByDepth.entries())
                .sort(([a], [b]) => a - b)
                .map(([depth, objects]) => (
                  <div key={depth} style={{ marginBottom: theme.spacing.sm }}>
                    <div
                      style={{
                        fontWeight: 600,
                        fontSize: "0.85rem",
                        color: theme.colors.textSecondary,
                        marginBottom: theme.spacing.xs,
                        borderBottom: `1px solid ${theme.colors.borderLight}`,
                        paddingBottom: theme.spacing.xs,
                      }}
                    >
                      Depth {depth}
                    </div>
                    {objects.map((obj) => (
                      <div
                        key={obj.id}
                        style={{
                          padding: `${theme.spacing.xs} ${theme.spacing.sm}`,
                          paddingLeft: `${depth * 16 + 8}px`,
                          fontSize: "0.9rem",
                          display: "flex",
                          gap: theme.spacing.sm,
                          alignItems: "baseline",
                        }}
                      >
                        <code style={{ color: theme.colors.textMuted, fontSize: "0.8rem" }}>
                          {obj.level}
                        </code>
                        <span>{obj.heading ?? "(no heading)"}</span>
                        {obj.link_type && (
                          <span
                            style={{
                              fontSize: "0.75rem",
                              background: theme.colors.bgCode,
                              padding: "1px 6px",
                              borderRadius: 3,
                              color: theme.colors.textSecondary,
                            }}
                          >
                            {obj.link_type}
                          </span>
                        )}
                      </div>
                    ))}
                  </div>
                ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}
