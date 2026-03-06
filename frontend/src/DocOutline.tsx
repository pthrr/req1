import type { ReqObject } from "./api/client";
import { theme } from "./theme";

interface DocOutlineProps {
  objects: ReqObject[];
  activeObjectId: string | null;
  onSelect: (id: string) => void;
}

export function DocOutline({ objects, activeObjectId, onSelect }: DocOutlineProps) {
  const headingObjects = objects.filter(
    (o) => o.heading != null && o.heading.trim() !== "",
  );

  return (
    <div
      style={{
        width: 250,
        minWidth: 250,
        borderRight: `1px solid ${theme.colors.borderLight}`,
        overflowY: "auto",
        padding: theme.spacing.sm,
        fontSize: "0.85rem",
      }}
    >
      <div style={{ fontWeight: 600, marginBottom: theme.spacing.sm, color: theme.colors.textSecondary }}>
        Outline
      </div>
      {headingObjects.map((obj) => {
        const depth = obj.level.split(".").length - 1;
        const isActive = obj.id === activeObjectId;
        return (
          <div
            key={obj.id}
            onClick={() => onSelect(obj.id)}
            style={{
              paddingLeft: `${depth * 16}px`,
              padding: `3px 6px 3px ${depth * 16 + 6}px`,
              cursor: "pointer",
              borderRadius: theme.borderRadius,
              background: isActive ? theme.colors.primary : "transparent",
              color: isActive ? "#fff" : theme.colors.text,
              marginBottom: 1,
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
            title={`${obj.level} ${obj.heading}`}
          >
            <span style={{ color: isActive ? "rgba(255,255,255,0.7)" : theme.colors.textMuted, marginRight: 4 }}>
              {obj.level}
            </span>
            {obj.heading}
          </div>
        );
      })}
      {headingObjects.length === 0 && (
        <div style={{ color: theme.colors.textMuted, fontStyle: "italic" }}>
          No headings
        </div>
      )}
    </div>
  );
}
