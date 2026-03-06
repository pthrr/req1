import { useCallback, useEffect, useState } from "react";
import { api, type LifecycleModel } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
}

interface StateEntry {
  name: string;
  color: string;
  description: string;
}

interface TransitionEntry {
  from: string;
  to: string;
}

export function LifecyclePanel({ moduleId }: Props) {
  const [models, setModels] = useState<LifecycleModel[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  // Form state
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [initialState, setInitialState] = useState("new");
  const [states, setStates] = useState<StateEntry[]>([]);
  const [transitions, setTransitions] = useState<TransitionEntry[]>([]);

  const fetchModels = useCallback(async () => {
    try {
      const data = await api.listLifecycleModels(moduleId);
      setModels(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load lifecycle models");
    }
  }, [moduleId]);

  useEffect(() => {
    fetchModels();
  }, [fetchModels]);

  const resetForm = () => {
    setName("");
    setDescription("");
    setInitialState("new");
    setStates([]);
    setTransitions([]);
  };

  const loadModel = (model: LifecycleModel) => {
    setName(model.name);
    setDescription(model.description ?? "");
    setInitialState(model.initial_state);
    setStates(
      (model.states as StateEntry[]).map((s) => ({
        name: s.name,
        color: s.color ?? "#666",
        description: s.description ?? "",
      })),
    );
    setTransitions(model.transitions as TransitionEntry[]);
  };

  const handleCreate = async () => {
    try {
      await api.createLifecycleModel(moduleId, {
        name,
        description: description || undefined,
        initial_state: initialState,
        states: states.map((s) => ({ name: s.name, color: s.color || undefined, description: s.description || undefined })),
        transitions,
      });
      resetForm();
      setCreating(false);
      fetchModels();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create lifecycle model");
    }
  };

  const handleUpdate = async () => {
    if (!editing) return;
    try {
      await api.updateLifecycleModel(moduleId, editing, {
        name,
        description: description || undefined,
        initial_state: initialState,
        states: states.map((s) => ({ name: s.name, color: s.color || undefined, description: s.description || undefined })),
        transitions,
      });
      setEditing(null);
      resetForm();
      fetchModels();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update lifecycle model");
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm("Delete this lifecycle model?")) return;
    try {
      await api.deleteLifecycleModel(moduleId, id);
      if (editing === id) {
        setEditing(null);
        resetForm();
      }
      fetchModels();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete lifecycle model");
    }
  };

  const isEditing = creating || editing != null;

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: theme.spacing.md }}>
        <h3 style={{ margin: 0 }}>Lifecycle Models</h3>
        {!isEditing && (
          <button
            onClick={() => {
              resetForm();
              setCreating(true);
            }}
            style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
          >
            New Model
          </button>
        )}
      </div>

      {error && (
        <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>{error}</div>
      )}

      {/* List */}
      {!isEditing && (
        <div style={{ display: "flex", flexDirection: "column", gap: theme.spacing.sm }}>
          {models.length === 0 && (
            <div style={{ color: theme.colors.textMuted }}>No lifecycle models defined.</div>
          )}
          {models.map((m) => (
            <div
              key={m.id}
              style={{
                border: `1px solid ${theme.colors.borderLight}`,
                borderRadius: theme.borderRadius,
                padding: theme.spacing.md,
              }}
            >
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                  <strong>{m.name}</strong>
                  {m.description && (
                    <span style={{ color: theme.colors.textMuted, marginLeft: theme.spacing.sm }}>
                      — {m.description}
                    </span>
                  )}
                </div>
                <div style={{ display: "flex", gap: theme.spacing.sm }}>
                  <button
                    onClick={() => {
                      setEditing(m.id);
                      loadModel(m);
                    }}
                    style={{ padding: "4px 10px", fontSize: "0.85rem" }}
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDelete(m.id)}
                    style={{ padding: "4px 10px", fontSize: "0.85rem" }}
                  >
                    Delete
                  </button>
                </div>
              </div>
              <div style={{ display: "flex", gap: theme.spacing.sm, marginTop: theme.spacing.sm, flexWrap: "wrap" }}>
                {(m.states as StateEntry[]).map((s) => (
                  <span
                    key={s.name}
                    style={{
                      display: "inline-block",
                      padding: "2px 10px",
                      borderRadius: "10px",
                      fontSize: "0.8rem",
                      fontWeight: 600,
                      color: "#fff",
                      background: s.color ?? "#666",
                    }}
                  >
                    {s.name}
                    {s.name === m.initial_state && " (initial)"}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Create / Edit form */}
      {isEditing && (
        <div style={{ border: `1px solid ${theme.colors.border}`, borderRadius: theme.borderRadius, padding: theme.spacing.md }}>
          <h4 style={{ margin: `0 0 ${theme.spacing.sm}` }}>
            {creating ? "Create Lifecycle Model" : "Edit Lifecycle Model"}
          </h4>

          <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: theme.spacing.sm }}>
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Model name"
              style={{ padding: theme.spacing.sm, flex: 1 }}
            />
            <input
              value={initialState}
              onChange={(e) => setInitialState(e.target.value)}
              placeholder="Initial state"
              style={{ padding: theme.spacing.sm, width: 150 }}
            />
          </div>

          <input
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Description"
            style={{ padding: theme.spacing.sm, width: "100%", marginBottom: theme.spacing.sm, boxSizing: "border-box" }}
          />

          {/* States */}
          <h5 style={{ margin: `${theme.spacing.sm} 0 4px` }}>States</h5>
          {states.map((s, i) => (
            <div key={i} style={{ display: "flex", gap: "4px", marginBottom: "4px", alignItems: "center" }}>
              <input
                value={s.name}
                onChange={(e) => {
                  const updated = [...states];
                  updated[i] = { ...updated[i], name: e.target.value };
                  setStates(updated);
                }}
                placeholder="State name"
                style={{ padding: "4px", flex: 1 }}
              />
              <input
                type="color"
                value={s.color || "#666666"}
                onChange={(e) => {
                  const updated = [...states];
                  updated[i] = { ...updated[i], color: e.target.value };
                  setStates(updated);
                }}
                style={{ width: 36, height: 28, padding: 0, border: "none", cursor: "pointer" }}
              />
              <button
                onClick={() => setStates(states.filter((_, j) => j !== i))}
                style={{ padding: "4px 8px", fontSize: "0.8rem" }}
              >
                X
              </button>
            </div>
          ))}
          <button
            onClick={() => setStates([...states, { name: "", color: "#666666", description: "" }])}
            style={{ padding: "4px 10px", fontSize: "0.85rem", marginBottom: theme.spacing.sm }}
          >
            + Add State
          </button>

          {/* Transitions */}
          <h5 style={{ margin: `${theme.spacing.sm} 0 4px` }}>Transitions</h5>
          {transitions.map((t, i) => (
            <div key={i} style={{ display: "flex", gap: "4px", marginBottom: "4px", alignItems: "center" }}>
              <select
                value={t.from}
                onChange={(e) => {
                  const updated = [...transitions];
                  updated[i] = { ...updated[i], from: e.target.value };
                  setTransitions(updated);
                }}
                style={{ padding: "4px", flex: 1 }}
              >
                <option value="">From...</option>
                {states.map((s) => (
                  <option key={s.name} value={s.name}>
                    {s.name}
                  </option>
                ))}
              </select>
              <span style={{ color: theme.colors.textMuted }}>&rarr;</span>
              <select
                value={t.to}
                onChange={(e) => {
                  const updated = [...transitions];
                  updated[i] = { ...updated[i], to: e.target.value };
                  setTransitions(updated);
                }}
                style={{ padding: "4px", flex: 1 }}
              >
                <option value="">To...</option>
                {states.map((s) => (
                  <option key={s.name} value={s.name}>
                    {s.name}
                  </option>
                ))}
              </select>
              <button
                onClick={() => setTransitions(transitions.filter((_, j) => j !== i))}
                style={{ padding: "4px 8px", fontSize: "0.8rem" }}
              >
                X
              </button>
            </div>
          ))}
          <button
            onClick={() => setTransitions([...transitions, { from: "", to: "" }])}
            style={{ padding: "4px 10px", fontSize: "0.85rem", marginBottom: theme.spacing.md }}
          >
            + Add Transition
          </button>

          <div style={{ display: "flex", gap: theme.spacing.sm }}>
            <button
              onClick={creating ? handleCreate : handleUpdate}
              style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
            >
              {creating ? "Create" : "Save"}
            </button>
            <button
              onClick={() => {
                setCreating(false);
                setEditing(null);
                resetForm();
              }}
              style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
