import { useEffect, useState } from "react";
import { api, type ReqObject } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  objects: ReqObject[];
}

interface FeedEntry {
  objectId: string;
  objectHeading: string | null;
  version: number;
  changeType: string;
  changedAt: string;
  changedBy: string | null;
}

export function ActivityFeed({ moduleId, objects }: Props) {
  const [entries, setEntries] = useState<FeedEntry[]>([]);
  const [expanded, setExpanded] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!expanded || objects.length === 0) return;
    let cancelled = false;
    setLoading(true);

    (async () => {
      const feed: FeedEntry[] = [];
      // Fetch history for the most recently updated objects (up to 10)
      const sorted = [...objects].sort(
        (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime(),
      );
      const recent = sorted.slice(0, 10);

      for (const obj of recent) {
        try {
          const hist = await api.listObjectHistory(moduleId, obj.id);
          for (const h of hist.items) {
            feed.push({
              objectId: obj.id,
              objectHeading: obj.heading,
              version: h.version,
              changeType: h.change_type,
              changedAt: h.changed_at,
              changedBy: h.changed_by,
            });
          }
        } catch {
          // Skip
        }
      }

      if (!cancelled) {
        feed.sort((a, b) => new Date(b.changedAt).getTime() - new Date(a.changedAt).getTime());
        setEntries(feed.slice(0, 50));
        setLoading(false);
      }
    })();

    return () => { cancelled = true; };
  }, [expanded, moduleId, objects]);

  return (
    <div style={{ marginTop: theme.spacing.md }}>
      <button
        onClick={() => setExpanded((p) => !p)}
        style={{
          padding: `${theme.spacing.sm} ${theme.spacing.md}`,
          fontSize: "0.85rem",
          background: "none",
          border: `1px solid ${theme.colors.borderLight}`,
          borderRadius: theme.borderRadius,
          cursor: "pointer",
          display: "flex",
          alignItems: "center",
          gap: theme.spacing.sm,
        }}
      >
        <span style={{ fontSize: "0.7em", transform: expanded ? "rotate(0deg)" : "rotate(-90deg)", transition: "transform 0.15s" }}>
          {"\u25BC"}
        </span>
        Activity Feed
      </button>

      {expanded && (
        <div
          style={{
            marginTop: theme.spacing.sm,
            border: `1px solid ${theme.colors.borderLight}`,
            borderRadius: theme.borderRadius,
            maxHeight: 300,
            overflow: "auto",
          }}
        >
          {loading && (
            <div style={{ padding: theme.spacing.md, color: theme.colors.textMuted, textAlign: "center" }}>
              Loading activity...
            </div>
          )}

          {!loading && entries.length === 0 && (
            <div style={{ padding: theme.spacing.md, color: theme.colors.textMuted, textAlign: "center" }}>
              No recent activity.
            </div>
          )}

          {!loading && entries.map((entry, i) => (
            <div
              key={`${entry.objectId}-${entry.version}-${i}`}
              style={{
                padding: `${theme.spacing.xs} ${theme.spacing.sm}`,
                borderBottom: `1px solid ${theme.colors.borderLight}`,
                fontSize: "0.82rem",
                display: "flex",
                gap: theme.spacing.sm,
                alignItems: "baseline",
              }}
            >
              <span style={{ color: theme.colors.textMuted, fontSize: "0.75rem", whiteSpace: "nowrap" }}>
                {new Date(entry.changedAt).toLocaleString()}
              </span>
              <span style={{
                padding: "1px 6px",
                borderRadius: 3,
                fontSize: "0.75rem",
                background: entry.changeType === "create" ? "#d4edda" : entry.changeType === "delete" ? "#f8d7da" : "#fff3cd",
                color: entry.changeType === "create" ? "#155724" : entry.changeType === "delete" ? "#721c24" : "#856404",
              }}>
                {entry.changeType}
              </span>
              <span style={{ fontWeight: 500 }}>
                {entry.objectHeading ?? entry.objectId.slice(0, 8) + "..."}
              </span>
              <span style={{ color: theme.colors.textMuted }}>
                v{entry.version}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
