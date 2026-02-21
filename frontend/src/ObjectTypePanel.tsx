import { useCallback, useEffect, useState } from "react";
import { api, type ObjectType } from "./api/client";

const CLASSIFICATIONS = ["normative", "informative", "heading"] as const;

interface Props {
  moduleId: string;
}

export function ObjectTypePanel({ moduleId }: Props) {
  const [types, setTypes] = useState<ObjectType[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [defaultClassification, setDefaultClassification] = useState("normative");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editDescription, setEditDescription] = useState("");
  const [editClassification, setEditClassification] = useState("normative");

  const fetchTypes = useCallback(async () => {
    try {
      const data = await api.listObjectTypes(moduleId);
      setTypes(data.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load object types");
    }
  }, [moduleId]);

  useEffect(() => {
    fetchTypes();
  }, [fetchTypes]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    try {
      await api.createObjectType({
        module_id: moduleId,
        name: name.trim(),
        description: description.trim() || undefined,
        default_classification: defaultClassification,
      });
      setName("");
      setDescription("");
      setDefaultClassification("normative");
      await fetchTypes();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create object type");
    }
  };

  const startEdit = (t: ObjectType) => {
    setEditingId(t.id);
    setEditName(t.name);
    setEditDescription(t.description ?? "");
    setEditClassification(t.default_classification);
  };

  const cancelEdit = () => {
    setEditingId(null);
  };

  const handleUpdate = async (t: ObjectType) => {
    try {
      const payload: { name?: string; description?: string; default_classification?: string } = {};
      if (editName.trim() && editName.trim() !== t.name) payload.name = editName.trim();
      if (editDescription.trim() !== (t.description ?? ""))
        payload.description = editDescription.trim();
      if (editClassification !== t.default_classification)
        payload.default_classification = editClassification;
      if (Object.keys(payload).length > 0) {
        await api.updateObjectType(t.id, payload);
      }
      setEditingId(null);
      await fetchTypes();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update object type");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteObjectType(id);
      if (editingId === id) setEditingId(null);
      await fetchTypes();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete object type");
    }
  };

  return (
    <div>
      {error && (
        <div style={{ color: "red", marginBottom: "0.5rem" }}>{error}</div>
      )}

      <form
        onSubmit={handleCreate}
        style={{ display: "flex", gap: "0.5rem", marginBottom: "1rem", flexWrap: "wrap" }}
      >
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Type name"
          style={{ padding: "0.4rem" }}
        />
        <input
          type="text"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Description (optional)"
          style={{ padding: "0.4rem", flex: 1 }}
        />
        <select
          value={defaultClassification}
          onChange={(e) => setDefaultClassification(e.target.value)}
          style={{ padding: "0.4rem" }}
        >
          {CLASSIFICATIONS.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>
        <button type="submit" style={{ padding: "0.4rem 0.8rem" }}>
          Add
        </button>
      </form>

      <table style={{ width: "100%", borderCollapse: "collapse" }}>
        <thead>
          <tr>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: "0.4rem" }}>
              Name
            </th>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: "0.4rem" }}>
              Description
            </th>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: "0.4rem" }}>
              Default Classification
            </th>
            <th style={{ borderBottom: "2px solid #ccc", padding: "0.4rem" }} />
          </tr>
        </thead>
        <tbody>
          {types.map((t) => (
            <tr key={t.id}>
              {editingId === t.id ? (
                <>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    <input
                      value={editName}
                      onChange={(e) => setEditName(e.target.value)}
                      style={{ width: "100%", padding: "0.2rem" }}
                    />
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    <input
                      value={editDescription}
                      onChange={(e) => setEditDescription(e.target.value)}
                      style={{ width: "100%", padding: "0.2rem" }}
                    />
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    <select
                      value={editClassification}
                      onChange={(e) => setEditClassification(e.target.value)}
                      style={{ padding: "0.2rem" }}
                    >
                      {CLASSIFICATIONS.map((c) => (
                        <option key={c} value={c}>
                          {c}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee", whiteSpace: "nowrap" }}>
                    <button onClick={() => handleUpdate(t)} style={{ marginRight: "0.25rem" }}>
                      Save
                    </button>
                    <button onClick={cancelEdit}>Cancel</button>
                  </td>
                </>
              ) : (
                <>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {t.name}
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {t.description ?? "\u2014"}
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {t.default_classification}
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee", whiteSpace: "nowrap" }}>
                    <button onClick={() => startEdit(t)} style={{ marginRight: "0.25rem" }}>
                      Edit
                    </button>
                    <button onClick={() => handleDelete(t.id)}>Delete</button>
                  </td>
                </>
              )}
            </tr>
          ))}
          {types.length === 0 && (
            <tr>
              <td
                colSpan={4}
                style={{ padding: "0.75rem", textAlign: "center", color: "#999" }}
              >
                No object types yet.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
