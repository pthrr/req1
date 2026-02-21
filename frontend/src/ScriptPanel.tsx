import { useCallback, useEffect, useState } from "react";
import { api, type Script } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
}

const SCRIPT_TYPES = ["trigger", "layout", "action"] as const;
const HOOK_POINTS = ["pre_save", "post_save", "pre_delete", "post_delete"] as const;

export function ScriptPanel({ moduleId }: Props) {
  const [scripts, setScripts] = useState<Script[]>([]);
  const [selected, setSelected] = useState<Script | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [testOutput, setTestOutput] = useState<string | null>(null);

  // Create form state
  const [newName, setNewName] = useState("");
  const [newType, setNewType] = useState<string>("trigger");
  const [newHook, setNewHook] = useState<string>("pre_save");
  const [newSource, setNewSource] = useState("");

  // Edit state
  const [editSource, setEditSource] = useState("");
  const [editName, setEditName] = useState("");

  const fetchScripts = useCallback(async () => {
    try {
      const data = await api.listScripts(moduleId);
      setScripts(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load scripts");
    }
  }, [moduleId]);

  useEffect(() => {
    fetchScripts();
  }, [fetchScripts]);

  useEffect(() => {
    if (selected) {
      setEditSource(selected.source_code);
      setEditName(selected.name);
    }
  }, [selected]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim() || !newSource.trim()) return;
    try {
      await api.createScript(moduleId, {
        name: newName.trim(),
        script_type: newType,
        hook_point: newType === "trigger" ? newHook : undefined,
        source_code: newSource,
      });
      setNewName("");
      setNewSource("");
      fetchScripts();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create script");
    }
  };

  const handleSave = async () => {
    if (!selected) return;
    try {
      const updated = await api.updateScript(moduleId, selected.id, {
        name: editName,
        source_code: editSource,
      });
      setSelected(updated);
      fetchScripts();
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save script");
    }
  };

  const handleToggleEnabled = async (script: Script) => {
    try {
      const updated = await api.updateScript(moduleId, script.id, {
        enabled: !script.enabled,
      });
      if (selected?.id === script.id) setSelected(updated);
      fetchScripts();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to toggle script");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteScript(moduleId, id);
      if (selected?.id === id) setSelected(null);
      fetchScripts();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete script");
    }
  };

  const handleTest = async () => {
    if (!selected) return;
    setTestOutput(null);
    try {
      // For triggers/layouts, provide a minimal mock object
      const mockObject =
        selected.script_type !== "action"
          ? {
              id: "00000000-0000-0000-0000-000000000000",
              heading: "Test Object",
              body: "Test body",
              level: "1",
              classification: "normative",
              attributes: {},
              version: 1,
            }
          : undefined;

      const result = await api.testScript(moduleId, selected.id, {
        object: mockObject,
      });
      setTestOutput(JSON.stringify(result, null, 2));
    } catch (err) {
      setTestOutput(
        `Error: ${err instanceof Error ? err.message : "Test failed"}`,
      );
    }
  };

  const handleExecute = async () => {
    if (!selected || selected.script_type !== "action") return;
    setTestOutput(null);
    try {
      const result = await api.executeScript(moduleId, selected.id);
      setTestOutput(
        `Executed successfully.\nMutations applied: ${result.mutations_applied}\nOutput:\n${result.output.join("\n")}`,
      );
      setError(null);
    } catch (err) {
      setTestOutput(
        `Error: ${err instanceof Error ? err.message : "Execution failed"}`,
      );
    }
  };

  return (
    <div style={{ display: "flex", gap: theme.spacing.md, minHeight: 400 }}>
      {/* Left: Script list + create form */}
      <div style={{ width: 280, flexShrink: 0 }}>
        <h3 style={{ marginTop: 0 }}>Scripts</h3>

        {error && (
          <div
            style={{
              color: theme.colors.error,
              marginBottom: theme.spacing.sm,
              fontSize: "0.85rem",
            }}
          >
            {error}
          </div>
        )}

        <div
          style={{
            border: `1px solid ${theme.colors.borderLight}`,
            borderRadius: 4,
            maxHeight: 250,
            overflowY: "auto",
            marginBottom: theme.spacing.md,
          }}
        >
          {scripts.length === 0 && (
            <div
              style={{
                padding: theme.spacing.md,
                color: theme.colors.tabInactive,
                textAlign: "center",
              }}
            >
              No scripts
            </div>
          )}
          {scripts.map((s) => (
            <div
              key={s.id}
              style={{
                padding: `${theme.spacing.sm} ${theme.spacing.md}`,
                cursor: "pointer",
                background:
                  selected?.id === s.id ? theme.colors.bgHover : "transparent",
                borderBottom: `1px solid ${theme.colors.borderLight}`,
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                gap: theme.spacing.sm,
              }}
              onClick={() => setSelected(s)}
            >
              <div style={{ minWidth: 0 }}>
                <div
                  style={{
                    fontWeight: 500,
                    opacity: s.enabled ? 1 : 0.5,
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {s.name}
                </div>
                <div
                  style={{
                    fontSize: "0.75rem",
                    color: theme.colors.tabInactive,
                  }}
                >
                  {s.script_type}
                  {s.hook_point ? ` (${s.hook_point})` : ""}
                </div>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleDelete(s.id);
                }}
                style={{ fontSize: "0.75rem", flexShrink: 0 }}
              >
                Del
              </button>
            </div>
          ))}
        </div>

        {/* Create form */}
        <form onSubmit={handleCreate}>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: theme.spacing.sm,
            }}
          >
            <input
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Script name"
              style={{ padding: theme.spacing.sm }}
            />
            <select
              value={newType}
              onChange={(e) => setNewType(e.target.value)}
              style={{ padding: theme.spacing.sm }}
            >
              {SCRIPT_TYPES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
            {newType === "trigger" && (
              <select
                value={newHook}
                onChange={(e) => setNewHook(e.target.value)}
                style={{ padding: theme.spacing.sm }}
              >
                {HOOK_POINTS.map((h) => (
                  <option key={h} value={h}>
                    {h}
                  </option>
                ))}
              </select>
            )}
            <textarea
              value={newSource}
              onChange={(e) => setNewSource(e.target.value)}
              placeholder="-- Lua source code"
              rows={4}
              style={{
                padding: theme.spacing.sm,
                fontFamily: "monospace",
                fontSize: "0.85rem",
              }}
            />
            <button type="submit" style={{ padding: theme.spacing.sm }}>
              Create Script
            </button>
          </div>
        </form>
      </div>

      {/* Right: Editor */}
      <div style={{ flex: 1, minWidth: 0 }}>
        {selected ? (
          <div>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: theme.spacing.md,
                marginBottom: theme.spacing.md,
              }}
            >
              <input
                type="text"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                style={{
                  padding: theme.spacing.sm,
                  fontWeight: 600,
                  fontSize: "1rem",
                  flex: 1,
                }}
              />
              <span
                style={{
                  fontSize: "0.85rem",
                  color: theme.colors.tabInactive,
                }}
              >
                {selected.script_type}
                {selected.hook_point ? ` / ${selected.hook_point}` : ""}
              </span>
              <label
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 4,
                  whiteSpace: "nowrap",
                }}
              >
                <input
                  type="checkbox"
                  checked={selected.enabled}
                  onChange={() => handleToggleEnabled(selected)}
                />
                Enabled
              </label>
            </div>

            <textarea
              value={editSource}
              onChange={(e) => setEditSource(e.target.value)}
              rows={16}
              style={{
                width: "100%",
                fontFamily: "monospace",
                fontSize: "0.85rem",
                padding: theme.spacing.sm,
                boxSizing: "border-box",
                resize: "vertical",
              }}
            />

            <div
              style={{
                display: "flex",
                gap: theme.spacing.sm,
                marginTop: theme.spacing.sm,
              }}
            >
              <button onClick={handleSave} style={{ padding: theme.spacing.sm }}>
                Save
              </button>
              <button onClick={handleTest} style={{ padding: theme.spacing.sm }}>
                Test (dry-run)
              </button>
              {selected.script_type === "action" && (
                <button
                  onClick={handleExecute}
                  style={{ padding: theme.spacing.sm }}
                >
                  Execute
                </button>
              )}
            </div>

            {testOutput && (
              <pre
                style={{
                  marginTop: theme.spacing.md,
                  padding: theme.spacing.md,
                  background: theme.colors.bgCode,
                  borderRadius: 4,
                  fontSize: "0.8rem",
                  overflow: "auto",
                  maxHeight: 200,
                  whiteSpace: "pre-wrap",
                }}
              >
                {testOutput}
              </pre>
            )}
          </div>
        ) : (
          <div
            style={{
              color: theme.colors.tabInactive,
              padding: theme.spacing.lg,
              textAlign: "center",
            }}
          >
            Select a script to edit, or create a new one.
          </div>
        )}
      </div>
    </div>
  );
}
