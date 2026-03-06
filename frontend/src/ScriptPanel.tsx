import { useCallback, useEffect, useState } from "react";
import { api, type Script, type ScriptExecution } from "./api/client";
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
  const [newPriority, setNewPriority] = useState(100);

  // Edit state
  const [editSource, setEditSource] = useState("");
  const [editName, setEditName] = useState("");
  const [editPriority, setEditPriority] = useState(100);
  const [editCron, setEditCron] = useState("");

  // Execution history
  const [executions, setExecutions] = useState<ScriptExecution[]>([]);

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
      setEditPriority(selected.priority);
      setEditCron(selected.cron_expression ?? "");
      // Fetch executions for action scripts
      if (selected.script_type === "action") {
        api.listScriptExecutions(moduleId, selected.id)
          .then((data) => setExecutions(data.items))
          .catch(() => setExecutions([]));
      } else {
        setExecutions([]);
      }
    }
  }, [selected, moduleId]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim() || !newSource.trim()) return;
    try {
      await api.createScript(moduleId, {
        name: newName.trim(),
        script_type: newType,
        hook_point: newType === "trigger" ? newHook : undefined,
        source_code: newSource,
        priority: newPriority,
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
        priority: editPriority,
        ...(selected.script_type === "action" && editCron ? { cron_expression: editCron } : {}),
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
                  {" | P:" + s.priority}
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
            <input
              type="number"
              value={newPriority}
              onChange={(e) => setNewPriority(Number(e.target.value))}
              placeholder="Priority (default 100)"
              style={{ padding: theme.spacing.sm }}
            />
            <textarea
              value={newSource}
              onChange={(e) => setNewSource(e.target.value)}
              placeholder="// JavaScript source code"
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
              <input
                type="number"
                value={editPriority}
                onChange={(e) => setEditPriority(Number(e.target.value))}
                title="Priority (lower = runs first)"
                style={{ padding: theme.spacing.sm, width: 70 }}
              />
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

            {selected.script_type === "action" && (
              <div style={{ marginTop: theme.spacing.sm, display: "flex", alignItems: "center", gap: theme.spacing.sm }}>
                <label style={{ fontSize: "0.85rem", whiteSpace: "nowrap" }}>CRON:</label>
                <input
                  type="text"
                  value={editCron}
                  onChange={(e) => setEditCron(e.target.value)}
                  placeholder="e.g. 0 0 * * * * (sec min hour dom mon dow)"
                  style={{ padding: theme.spacing.sm, flex: 1, fontFamily: "monospace", fontSize: "0.85rem" }}
                />
                {selected.last_run_at && (
                  <span style={{ fontSize: "0.75rem", color: theme.colors.tabInactive }}>
                    Last: {new Date(selected.last_run_at).toLocaleString()}
                  </span>
                )}
                {selected.next_run_at && (
                  <span style={{ fontSize: "0.75rem", color: theme.colors.tabInactive }}>
                    Next: {new Date(selected.next_run_at).toLocaleString()}
                  </span>
                )}
              </div>
            )}

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

            {selected.script_type === "action" && executions.length > 0 && (
              <div style={{ marginTop: theme.spacing.lg }}>
                <h4 style={{ marginBottom: theme.spacing.sm }}>Execution History</h4>
                <div style={{ border: `1px solid ${theme.colors.borderLight}`, borderRadius: 4, maxHeight: 200, overflowY: "auto" }}>
                  <table style={{ width: "100%", fontSize: "0.8rem", borderCollapse: "collapse" }}>
                    <thead>
                      <tr style={{ background: theme.colors.bgHover }}>
                        <th style={{ padding: 4, textAlign: "left" }}>Status</th>
                        <th style={{ padding: 4, textAlign: "left" }}>Started</th>
                        <th style={{ padding: 4, textAlign: "left" }}>Duration</th>
                        <th style={{ padding: 4, textAlign: "left" }}>Error</th>
                      </tr>
                    </thead>
                    <tbody>
                      {executions.map((ex) => (
                        <tr key={ex.id} style={{ borderTop: `1px solid ${theme.colors.borderLight}` }}>
                          <td style={{ padding: 4 }}>
                            <span style={{
                              padding: "2px 6px",
                              borderRadius: 3,
                              fontSize: "0.75rem",
                              fontWeight: 600,
                              background: ex.status === "success" ? "#d4edda" : ex.status === "error" ? "#f8d7da" : "#fff3cd",
                              color: ex.status === "success" ? "#155724" : ex.status === "error" ? "#721c24" : "#856404",
                            }}>
                              {ex.status}
                            </span>
                          </td>
                          <td style={{ padding: 4 }}>{new Date(ex.started_at).toLocaleString()}</td>
                          <td style={{ padding: 4 }}>{ex.duration_ms != null ? `${ex.duration_ms}ms` : "-"}</td>
                          <td style={{ padding: 4, color: theme.colors.error }}>{ex.error_message ?? "-"}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
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
