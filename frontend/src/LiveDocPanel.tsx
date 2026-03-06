import { useCallback, useEffect, useRef, useState } from "react";
import type { ReqObject } from "./api/client";
import { api } from "./api/client";
import { DocOutline } from "./DocOutline";
import { DocBlock } from "./DocBlock";
import { theme } from "./theme";

interface LiveDocPanelProps {
  moduleId: string;
  objects: ReqObject[];
  onObjectsChanged: () => void;
}

export function LiveDocPanel({ moduleId, objects, onObjectsChanged }: LiveDocPanelProps) {
  const [editingObjectId, setEditingObjectId] = useState<string | null>(null);
  const [activeOutlineId, setActiveOutlineId] = useState<string | null>(null);
  const docAreaRef = useRef<HTMLDivElement>(null);

  // IntersectionObserver for scroll tracking
  useEffect(() => {
    const root = docAreaRef.current;
    if (!root) return;

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            const id = (entry.target as HTMLElement).dataset.objectId;
            if (id) setActiveOutlineId(id);
          }
        }
      },
      {
        root,
        rootMargin: "-20% 0px -60% 0px",
      },
    );

    const elements = root.querySelectorAll("[data-object-id]");
    elements.forEach((el) => observer.observe(el));

    return () => observer.disconnect();
  }, [objects, editingObjectId]);

  const handleOutlineSelect = useCallback((id: string) => {
    const el = document.getElementById(`doc-obj-${id}`);
    el?.scrollIntoView({ behavior: "smooth" });
  }, []);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: 600 }}>
      {/* Toolbar */}
      <div
        style={{
          display: "flex",
          gap: theme.spacing.sm,
          padding: theme.spacing.sm,
          borderBottom: `1px solid ${theme.colors.borderLight}`,
          marginBottom: theme.spacing.sm,
        }}
      >
        <button
          onClick={() => window.open(api.getPublishUrl(moduleId, "docx"), "_blank")}
          style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, fontSize: "0.85rem" }}
        >
          Export Word
        </button>
        <button
          onClick={() => window.open(api.getPublishUrl(moduleId, "pdf"), "_blank")}
          style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, fontSize: "0.85rem" }}
        >
          Export PDF
        </button>
        <button
          onClick={() => window.print()}
          style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, fontSize: "0.85rem" }}
        >
          Print
        </button>
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: "0.8rem", color: theme.colors.textMuted, alignSelf: "center" }}>
          Double-click any block to edit
        </span>
      </div>

      {/* Main area */}
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <DocOutline
          objects={objects}
          activeObjectId={activeOutlineId}
          onSelect={handleOutlineSelect}
        />

        <div
          ref={docAreaRef}
          style={{
            flex: 1,
            overflowY: "auto",
            padding: `${theme.spacing.md} ${theme.spacing.lg}`,
          }}
        >
          {objects.map((obj) => (
            <div
              key={obj.id}
              id={`doc-obj-${obj.id}`}
              data-object-id={obj.id}
            >
              <DocBlock
                moduleId={moduleId}
                object={obj}
                isEditing={editingObjectId === obj.id}
                onStartEdit={() => setEditingObjectId(obj.id)}
                onStopEdit={() => setEditingObjectId(null)}
                onSaved={onObjectsChanged}
              />
            </div>
          ))}
          {objects.length === 0 && (
            <div style={{ color: theme.colors.textMuted, textAlign: "center", padding: theme.spacing.xl }}>
              No objects in this module.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
