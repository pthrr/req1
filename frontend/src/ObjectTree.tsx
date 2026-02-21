import { useCallback, useMemo, useState } from "react";
import type { ReqObject } from "./api/client";
import { theme } from "./theme";

interface TreeNode {
  obj: ReqObject;
  children: TreeNode[];
}

interface Props {
  objects: ReqObject[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

function buildTree(objects: ReqObject[]): TreeNode[] {
  const byId = new Map<string, TreeNode>();
  const roots: TreeNode[] = [];

  // Create nodes
  for (const obj of objects) {
    byId.set(obj.id, { obj, children: [] });
  }

  // Link parents
  for (const obj of objects) {
    const node = byId.get(obj.id)!;
    if (obj.parent_id && byId.has(obj.parent_id)) {
      byId.get(obj.parent_id)!.children.push(node);
    } else {
      roots.push(node);
    }
  }

  // Sort by position
  const sortChildren = (nodes: TreeNode[]) => {
    nodes.sort((a, b) => a.obj.position - b.obj.position);
    for (const n of nodes) sortChildren(n.children);
  };
  sortChildren(roots);

  return roots;
}

function TreeNodeRow({
  node,
  depth,
  selectedId,
  onSelect,
  collapsedIds,
  onToggle,
}: {
  node: TreeNode;
  depth: number;
  selectedId: string | null;
  onSelect: (id: string) => void;
  collapsedIds: Set<string>;
  onToggle: (id: string) => void;
}) {
  const hasChildren = node.children.length > 0;
  const isCollapsed = collapsedIds.has(node.obj.id);
  const isSelected = selectedId === node.obj.id;

  return (
    <>
      <div
        onClick={() => onSelect(node.obj.id)}
        style={{
          padding: "3px 8px",
          paddingLeft: `${8 + depth * 16}px`,
          cursor: "pointer",
          background: isSelected ? "#e3f2fd" : "transparent",
          fontSize: "0.82rem",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          display: "flex",
          alignItems: "center",
          gap: "4px",
          borderBottom: `1px solid ${theme.colors.borderLight}`,
        }}
      >
        {hasChildren ? (
          <span
            onClick={(e) => {
              e.stopPropagation();
              onToggle(node.obj.id);
            }}
            style={{
              cursor: "pointer",
              userSelect: "none",
              width: "14px",
              textAlign: "center",
              fontSize: "0.7em",
              transition: "transform 0.15s",
              display: "inline-block",
              transform: isCollapsed ? "rotate(-90deg)" : "rotate(0deg)",
            }}
          >
            {"\u25BC"}
          </span>
        ) : (
          <span style={{ width: "14px", textAlign: "center", color: theme.colors.textMuted }}>
            {"\u00B7"}
          </span>
        )}
        <span style={{ color: theme.colors.textSecondary, fontWeight: 500 }}>
          {node.obj.level}
        </span>
        <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>
          {node.obj.heading ?? ""}
        </span>
      </div>
      {hasChildren && !isCollapsed &&
        node.children.map((child) => (
          <TreeNodeRow
            key={child.obj.id}
            node={child}
            depth={depth + 1}
            selectedId={selectedId}
            onSelect={onSelect}
            collapsedIds={collapsedIds}
            onToggle={onToggle}
          />
        ))}
    </>
  );
}

export function ObjectTree({ objects, selectedId, onSelect }: Props) {
  const [collapsedIds, setCollapsedIds] = useState<Set<string>>(new Set());
  const tree = useMemo(() => buildTree(objects), [objects]);

  const handleToggle = useCallback((id: string) => {
    setCollapsedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  return (
    <div
      style={{
        width: 240,
        minWidth: 240,
        borderRight: `1px solid ${theme.colors.sidebarBorder}`,
        overflowY: "auto",
        overflowX: "hidden",
        height: "100%",
      }}
    >
      <div
        style={{
          padding: "6px 8px",
          fontWeight: 600,
          fontSize: "0.8rem",
          color: theme.colors.textSecondary,
          borderBottom: `1px solid ${theme.colors.sidebarBorder}`,
        }}
      >
        Object Tree
      </div>
      {tree.map((node) => (
        <TreeNodeRow
          key={node.obj.id}
          node={node}
          depth={0}
          selectedId={selectedId}
          onSelect={onSelect}
          collapsedIds={collapsedIds}
          onToggle={handleToggle}
        />
      ))}
    </div>
  );
}
