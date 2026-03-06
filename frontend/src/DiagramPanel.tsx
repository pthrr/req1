import { useCallback, useEffect, useRef, useState } from "react";
import { api, type Diagram } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
}

const DIAGRAM_TYPES = [
  { value: "use_case", label: "Use Case" },
  { value: "sequence", label: "Sequence" },
  { value: "class", label: "Class" },
  { value: "flowchart", label: "Flowchart" },
  { value: "state", label: "State" },
  { value: "er", label: "ER" },
] as const;

const DEFAULT_TEMPLATES: Record<string, string> = {
  use_case: `graph LR
  Actor1((Actor)) --> UC1[Use Case 1]
  Actor1 --> UC2[Use Case 2]`,
  sequence: `sequenceDiagram
  participant A
  participant B
  A->>B: Request
  B-->>A: Response`,
  class: `classDiagram
  class Animal {
    +String name
    +makeSound()
  }`,
  flowchart: `flowchart TD
  A[Start] --> B{Decision}
  B -->|Yes| C[Action]
  B -->|No| D[End]`,
  state: `stateDiagram-v2
  [*] --> Draft
  Draft --> Review
  Review --> Approved
  Review --> Draft`,
  er: `erDiagram
  CUSTOMER ||--o{ ORDER : places
  ORDER ||--|{ LINE-ITEM : contains`,
};

export function DiagramPanel({ moduleId }: Props) {
  const [diagrams, setDiagrams] = useState<Diagram[]>([]);
  const [selected, setSelected] = useState<Diagram | null>(null);
  const [error, setError] = useState<string | null>(null);
  const previewRef = useRef<HTMLDivElement>(null);

  // Create form
  const [newName, setNewName] = useState("");
  const [newType, setNewType] = useState("use_case");

  // Edit state
  const [editName, setEditName] = useState("");
  const [editType, setEditType] = useState("");
  const [editSource, setEditSource] = useState("");
  const [editDescription, setEditDescription] = useState("");

  const fetchDiagrams = useCallback(async () => {
    try {
      const data = await api.listDiagrams(moduleId);
      setDiagrams(data.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load diagrams");
    }
  }, [moduleId]);

  useEffect(() => {
    fetchDiagrams();
  }, [fetchDiagrams]);

  useEffect(() => {
    if (selected) {
      setEditName(selected.name);
      setEditType(selected.diagram_type);
      setEditSource(selected.source_code);
      setEditDescription(selected.description ?? "");
    }
  }, [selected]);

  // Render mermaid preview
  useEffect(() => {
    if (!editSource || !previewRef.current) return;
    const container = previewRef.current;
    container.innerHTML = "";
    import("mermaid").then((mermaid) => {
      mermaid.default.initialize({ startOnLoad: false, theme: "default" });
      const id = `mermaid-${Date.now()}`;
      mermaid.default.render(id, editSource).then(({ svg }) => {
        container.innerHTML = svg;
      }).catch(() => {
        container.textContent = "Invalid Mermaid syntax";
      });
    }).catch(() => {
      container.textContent = "Mermaid not available";
    });
  }, [editSource]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim()) return;
    try {
      const created = await api.createDiagram(moduleId, {
        name: newName.trim(),
        diagram_type: newType,
        source_code: DEFAULT_TEMPLATES[newType] ?? "",
      });
      setNewName("");
      await fetchDiagrams();
      setSelected(created);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create diagram");
    }
  };

  const handleSave = async () => {
    if (!selected) return;
    try {
      const updated = await api.updateDiagram(moduleId, selected.id, {
        name: editName,
        diagram_type: editType,
        source_code: editSource,
        description: editDescription || undefined,
      });
      setSelected(updated);
      fetchDiagrams();
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save diagram");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteDiagram(moduleId, id);
      if (selected?.id === id) setSelected(null);
      fetchDiagrams();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete diagram");
    }
  };

  return (
    <div style={{ display: "flex", gap: theme.spacing.md, minHeight: 400 }}>
      {/* Left: List + create */}
      <div style={{ width: 260, flexShrink: 0 }}>
        <h3 style={{ marginTop: 0 }}>Diagrams</h3>
        {error && (
          <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm, fontSize: "0.85rem" }}>
            {error}
          </div>
        )}

        <div style={{ border: `1px solid ${theme.colors.borderLight}`, borderRadius: 4, maxHeight: 300, overflowY: "auto", marginBottom: theme.spacing.md }}>
          {diagrams.length === 0 && (
            <div style={{ padding: theme.spacing.md, color: theme.colors.tabInactive, textAlign: "center" }}>
              No diagrams
            </div>
          )}
          {diagrams.map((d) => (
            <div
              key={d.id}
              style={{
                padding: `${theme.spacing.sm} ${theme.spacing.md}`,
                cursor: "pointer",
                background: selected?.id === d.id ? theme.colors.bgHover : "transparent",
                borderBottom: `1px solid ${theme.colors.borderLight}`,
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
              }}
              onClick={() => setSelected(d)}
            >
              <div>
                <div style={{ fontWeight: 500 }}>{d.name}</div>
                <div style={{ fontSize: "0.75rem", color: theme.colors.tabInactive }}>{d.diagram_type}</div>
              </div>
              <button
                onClick={(e) => { e.stopPropagation(); handleDelete(d.id); }}
                style={{ fontSize: "0.75rem", flexShrink: 0 }}
              >
                Del
              </button>
            </div>
          ))}
        </div>

        <form onSubmit={handleCreate} style={{ display: "flex", flexDirection: "column", gap: theme.spacing.sm }}>
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="Diagram name"
            style={{ padding: theme.spacing.sm }}
          />
          <select value={newType} onChange={(e) => setNewType(e.target.value)} style={{ padding: theme.spacing.sm }}>
            {DIAGRAM_TYPES.map((t) => (
              <option key={t.value} value={t.value}>{t.label}</option>
            ))}
          </select>
          <button type="submit" style={{ padding: theme.spacing.sm }}>Create Diagram</button>
        </form>
      </div>

      {/* Right: Editor + Preview */}
      <div style={{ flex: 1, minWidth: 0 }}>
        {selected ? (
          <div>
            <div style={{ display: "flex", gap: theme.spacing.md, marginBottom: theme.spacing.md, alignItems: "center" }}>
              <input
                type="text"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                style={{ padding: theme.spacing.sm, fontWeight: 600, fontSize: "1rem", flex: 1 }}
              />
              <select value={editType} onChange={(e) => setEditType(e.target.value)} style={{ padding: theme.spacing.sm }}>
                {DIAGRAM_TYPES.map((t) => (
                  <option key={t.value} value={t.value}>{t.label}</option>
                ))}
              </select>
            </div>

            <input
              type="text"
              value={editDescription}
              onChange={(e) => setEditDescription(e.target.value)}
              placeholder="Description (optional)"
              style={{ padding: theme.spacing.sm, width: "100%", boxSizing: "border-box", marginBottom: theme.spacing.sm }}
            />

            <div style={{ display: "flex", gap: theme.spacing.md }}>
              <textarea
                value={editSource}
                onChange={(e) => setEditSource(e.target.value)}
                rows={14}
                style={{
                  flex: 1,
                  fontFamily: "monospace",
                  fontSize: "0.85rem",
                  padding: theme.spacing.sm,
                  boxSizing: "border-box",
                  resize: "vertical",
                }}
              />
              <div
                ref={previewRef}
                style={{
                  flex: 1,
                  border: `1px solid ${theme.colors.borderLight}`,
                  borderRadius: 4,
                  padding: theme.spacing.sm,
                  minHeight: 200,
                  overflow: "auto",
                  background: "#fff",
                }}
              />
            </div>

            <div style={{ marginTop: theme.spacing.sm }}>
              <button onClick={handleSave} style={{ padding: theme.spacing.sm }}>Save</button>
            </div>
          </div>
        ) : (
          <div style={{ color: theme.colors.tabInactive, padding: theme.spacing.lg, textAlign: "center" }}>
            Select a diagram to edit, or create a new one.
          </div>
        )}
      </div>
    </div>
  );
}
