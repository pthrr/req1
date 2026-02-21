import { wordDiff, type DiffSegment } from "./diff";

const COLORS = {
  added: { bg: "#d4edda", color: "#155724" },
  removed: { bg: "#f8d7da", color: "#721c24" },
  equal: { bg: "transparent", color: "inherit" },
};

function segmentStyle(seg: DiffSegment): React.CSSProperties {
  const c = COLORS[seg.type];
  return {
    background: c.bg,
    color: c.color,
    textDecoration: seg.type === "removed" ? "line-through" : "none",
    padding: seg.type !== "equal" ? "1px 2px" : undefined,
    borderRadius: seg.type !== "equal" ? 2 : undefined,
  };
}

interface InlineDiffProps {
  label: string;
  textA: string | null;
  textB: string | null;
}

export function InlineDiff({ label, textA, textB }: InlineDiffProps) {
  const a = textA ?? "";
  const b = textB ?? "";
  if (a === b) return null;

  const segments = wordDiff(a, b);

  return (
    <div style={{ marginBottom: "0.5rem" }}>
      <strong style={{ fontSize: "0.8rem", color: "#666" }}>{label}:</strong>
      <div style={{ padding: "4px 0", lineHeight: 1.6 }}>
        {segments.map((seg, i) => (
          <span key={i} style={segmentStyle(seg)}>
            {seg.text}
          </span>
        ))}
      </div>
    </div>
  );
}

interface AttributesDiffProps {
  attrsA: Record<string, unknown> | null;
  attrsB: Record<string, unknown> | null;
}

export function AttributesDiff({ attrsA, attrsB }: AttributesDiffProps) {
  const a = attrsA ?? {};
  const b = attrsB ?? {};
  const allKeys = [...new Set([...Object.keys(a), ...Object.keys(b)])].sort();
  const changedKeys = allKeys.filter(
    (k) => JSON.stringify(a[k]) !== JSON.stringify(b[k]),
  );

  if (changedKeys.length === 0) return null;

  return (
    <div style={{ marginBottom: "0.5rem" }}>
      <strong style={{ fontSize: "0.8rem", color: "#666" }}>Attributes:</strong>
      <table
        style={{
          borderCollapse: "collapse",
          fontSize: "0.8rem",
          marginTop: "4px",
          width: "100%",
        }}
      >
        <thead>
          <tr>
            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "2px 6px" }}>
              Key
            </th>
            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "2px 6px" }}>
              Before
            </th>
            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "2px 6px" }}>
              After
            </th>
          </tr>
        </thead>
        <tbody>
          {changedKeys.map((key) => (
            <tr key={key}>
              <td style={{ padding: "2px 6px", borderBottom: "1px solid #eee" }}>{key}</td>
              <td
                style={{
                  padding: "2px 6px",
                  borderBottom: "1px solid #eee",
                  background: COLORS.removed.bg,
                  color: COLORS.removed.color,
                }}
              >
                {a[key] !== undefined ? JSON.stringify(a[key]) : "-"}
              </td>
              <td
                style={{
                  padding: "2px 6px",
                  borderBottom: "1px solid #eee",
                  background: COLORS.added.bg,
                  color: COLORS.added.color,
                }}
              >
                {b[key] !== undefined ? JSON.stringify(b[key]) : "-"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
