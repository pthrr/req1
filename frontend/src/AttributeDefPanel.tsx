import { useCallback, useEffect, useState } from "react";
import { api, type AttributeDefinition } from "./api/client";

const DATA_TYPES = [
  "string",
  "integer",
  "float",
  "date",
  "bool",
  "enum",
  "rich_text",
  "user_ref",
] as const;

interface Props {
  moduleId: string;
  onDefsChanged?: () => void;
}

export function AttributeDefPanel({ moduleId, onDefsChanged }: Props) {
  const [defs, setDefs] = useState<AttributeDefinition[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [dataType, setDataType] = useState<string>("string");
  const [defaultValue, setDefaultValue] = useState("");
  const [enumValues, setEnumValues] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editDefault, setEditDefault] = useState("");
  const [editEnum, setEditEnum] = useState("");

  const fetchDefs = useCallback(async () => {
    try {
      const data = await api.listAttributeDefinitions(moduleId);
      setDefs(data.items);
      setError(null);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to load attribute definitions",
      );
    }
  }, [moduleId]);

  useEffect(() => {
    fetchDefs();
  }, [fetchDefs]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    try {
      const payload: {
        name: string;
        data_type: string;
        default_value?: string;
        enum_values?: string[];
      } = {
        name: name.trim(),
        data_type: dataType,
      };
      if (defaultValue.trim()) payload.default_value = defaultValue.trim();
      if (dataType === "enum" && enumValues.trim()) {
        payload.enum_values = enumValues
          .split(",")
          .map((v) => v.trim())
          .filter(Boolean);
      }
      await api.createAttributeDefinition(moduleId, payload);
      setName("");
      setDefaultValue("");
      setEnumValues("");
      fetchDefs();
      onDefsChanged?.();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to create attribute definition",
      );
    }
  };

  const startEdit = (d: AttributeDefinition) => {
    setEditingId(d.id);
    setEditName(d.name);
    setEditDefault(d.default_value ?? "");
    setEditEnum(
      Array.isArray(d.enum_values) ? d.enum_values.join(", ") : "",
    );
  };

  const cancelEdit = () => {
    setEditingId(null);
  };

  const handleUpdate = async (d: AttributeDefinition) => {
    try {
      const payload: {
        name?: string;
        default_value?: string;
        enum_values?: string[];
      } = {};
      if (editName.trim() && editName.trim() !== d.name) {
        payload.name = editName.trim();
      }
      if (editDefault.trim() !== (d.default_value ?? "")) {
        payload.default_value = editDefault.trim();
      }
      if (d.data_type === "enum" && editEnum.trim()) {
        const vals = editEnum
          .split(",")
          .map((v) => v.trim())
          .filter(Boolean);
        const existing = Array.isArray(d.enum_values) ? d.enum_values : [];
        if (JSON.stringify(vals) !== JSON.stringify(existing)) {
          payload.enum_values = vals;
        }
      }
      if (Object.keys(payload).length > 0) {
        await api.updateAttributeDefinition(moduleId, d.id, payload);
      }
      setEditingId(null);
      fetchDefs();
      onDefsChanged?.();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to update attribute definition",
      );
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteAttributeDefinition(moduleId, id);
      if (editingId === id) setEditingId(null);
      fetchDefs();
      onDefsChanged?.();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to delete attribute definition",
      );
    }
  };

  const formatEnumValues = (d: AttributeDefinition): string => {
    if (!d.enum_values) return "—";
    if (Array.isArray(d.enum_values)) return d.enum_values.join(", ");
    return JSON.stringify(d.enum_values);
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
          placeholder="Attribute name"
          style={{ padding: "0.4rem" }}
        />
        <select
          value={dataType}
          onChange={(e) => setDataType(e.target.value)}
          style={{ padding: "0.4rem" }}
        >
          {DATA_TYPES.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>
        <input
          type="text"
          value={defaultValue}
          onChange={(e) => setDefaultValue(e.target.value)}
          placeholder="Default value"
          style={{ padding: "0.4rem" }}
        />
        {dataType === "enum" && (
          <input
            type="text"
            value={enumValues}
            onChange={(e) => setEnumValues(e.target.value)}
            placeholder="Enum values (comma-separated)"
            style={{ padding: "0.4rem", flex: 1 }}
          />
        )}
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
              Type
            </th>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: "0.4rem" }}>
              Default
            </th>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: "0.4rem" }}>
              Enum Values
            </th>
            <th style={{ borderBottom: "2px solid #ccc", padding: "0.4rem" }} />
          </tr>
        </thead>
        <tbody>
          {defs.map((d) => (
            <tr key={d.id}>
              {editingId === d.id ? (
                <>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    <input
                      value={editName}
                      onChange={(e) => setEditName(e.target.value)}
                      style={{ width: "100%", padding: "0.2rem" }}
                    />
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    <code>{d.data_type}</code>
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    <input
                      value={editDefault}
                      onChange={(e) => setEditDefault(e.target.value)}
                      style={{ width: "100%", padding: "0.2rem" }}
                    />
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {d.data_type === "enum" ? (
                      <input
                        value={editEnum}
                        onChange={(e) => setEditEnum(e.target.value)}
                        style={{ width: "100%", padding: "0.2rem" }}
                      />
                    ) : (
                      "—"
                    )}
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee", whiteSpace: "nowrap" }}>
                    <button onClick={() => handleUpdate(d)} style={{ marginRight: "0.25rem" }}>
                      Save
                    </button>
                    <button onClick={cancelEdit}>Cancel</button>
                  </td>
                </>
              ) : (
                <>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {d.name}
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    <code>{d.data_type}</code>
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {d.default_value ?? "—"}
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {formatEnumValues(d)}
                  </td>
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee", whiteSpace: "nowrap" }}>
                    <button onClick={() => startEdit(d)} style={{ marginRight: "0.25rem" }}>
                      Edit
                    </button>
                    <button onClick={() => handleDelete(d.id)}>Delete</button>
                  </td>
                </>
              )}
            </tr>
          ))}
          {defs.length === 0 && (
            <tr>
              <td
                colSpan={5}
                style={{ padding: "0.75rem", textAlign: "center", color: "#999" }}
              >
                No attribute definitions yet.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
