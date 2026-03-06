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
  const [dependsOn, setDependsOn] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editDefault, setEditDefault] = useState("");
  const [editEnum, setEditEnum] = useState("");
  const [editDependsOn, setEditDependsOn] = useState("");
  const [editMapping, setEditMapping] = useState<Record<string, string[]>>({});
  const [mappingEditorId, setMappingEditorId] = useState<string | null>(null);

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
        depends_on?: string;
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
      if (dependsOn) payload.depends_on = dependsOn;
      await api.createAttributeDefinition(moduleId, payload);
      setName("");
      setDefaultValue("");
      setEnumValues("");
      setDependsOn("");
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
    setEditDependsOn(d.depends_on ?? "");
    setEditMapping(d.dependency_mapping ?? {});
  };

  const cancelEdit = () => {
    setEditingId(null);
  };

  const handleUpdate = async (d: AttributeDefinition) => {
    try {
      const payload: Record<string, unknown> = {};
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
      if (editDependsOn !== (d.depends_on ?? "")) {
        payload.depends_on = editDependsOn || null;
      }
      if (JSON.stringify(editMapping) !== JSON.stringify(d.dependency_mapping ?? {})) {
        payload.dependency_mapping = editMapping;
      }
      if (Object.keys(payload).length > 0) {
        await api.updateAttributeDefinition(moduleId, d.id, payload as Parameters<typeof api.updateAttributeDefinition>[2]);
      }
      setEditingId(null);
      setMappingEditorId(null);
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
        {dataType === "enum" && (
          <select
            value={dependsOn}
            onChange={(e) => setDependsOn(e.target.value)}
            style={{ padding: "0.4rem" }}
          >
            <option value="">(no dependency)</option>
            {defs.filter((d) => d.data_type === "enum").map((d) => (
              <option key={d.id} value={d.id}>{d.name}</option>
            ))}
          </select>
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
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: "0.4rem" }}>
              Depends On
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
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {d.data_type === "enum" ? (
                      <div>
                        <select
                          value={editDependsOn}
                          onChange={(e) => setEditDependsOn(e.target.value)}
                          style={{ width: "100%", padding: "0.2rem", marginBottom: "0.25rem" }}
                        >
                          <option value="">(none)</option>
                          {defs.filter((dd) => dd.data_type === "enum" && dd.id !== d.id).map((dd) => (
                            <option key={dd.id} value={dd.id}>{dd.name}</option>
                          ))}
                        </select>
                        {editDependsOn && (
                          <button
                            onClick={() => setMappingEditorId(mappingEditorId === d.id ? null : d.id)}
                            style={{ fontSize: "0.75rem", padding: "1px 6px" }}
                          >
                            {mappingEditorId === d.id ? "Hide" : "Edit"} Mapping
                          </button>
                        )}
                      </div>
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
                  <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                    {d.depends_on ? (defs.find((dd) => dd.id === d.depends_on)?.name ?? d.depends_on.slice(0, 8)) : "—"}
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
                colSpan={6}
                style={{ padding: "0.75rem", textAlign: "center", color: "#999" }}
              >
                No attribute definitions yet.
              </td>
            </tr>
          )}
        </tbody>
      </table>

      {/* Dependency Mapping Editor */}
      {mappingEditorId && editDependsOn && (() => {
        const parentDef = defs.find((d) => d.id === editDependsOn);
        const childDef = defs.find((d) => d.id === mappingEditorId);
        if (!parentDef || !childDef) return null;
        const parentValues = Array.isArray(parentDef.enum_values) ? parentDef.enum_values : [];
        const childValues = Array.isArray(childDef.enum_values) ? childDef.enum_values : [];

        return (
          <div style={{ marginTop: "1rem", padding: "1rem", border: "1px solid #ccc", borderRadius: "4px", background: "#fafafa" }}>
            <h4 style={{ margin: "0 0 0.5rem" }}>
              Dependency Mapping: {parentDef.name} → {childDef.name}
            </h4>
            <p style={{ fontSize: "0.85rem", color: "#666", margin: "0 0 0.75rem" }}>
              For each parent value, select which child values are allowed.
            </p>
            <table style={{ borderCollapse: "collapse", width: "100%" }}>
              <thead>
                <tr>
                  <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: "0.4rem" }}>
                    Parent: {parentDef.name}
                  </th>
                  <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: "0.4rem" }}>
                    Allowed: {childDef.name}
                  </th>
                </tr>
              </thead>
              <tbody>
                {parentValues.map((pv) => (
                  <tr key={pv}>
                    <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee", fontWeight: 600 }}>{pv}</td>
                    <td style={{ padding: "0.4rem", borderBottom: "1px solid #eee" }}>
                      <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
                        {childValues.map((cv) => {
                          const checked = (editMapping[pv] ?? []).includes(cv);
                          return (
                            <label key={cv} style={{ display: "flex", alignItems: "center", gap: "0.2rem", fontSize: "0.85rem" }}>
                              <input
                                type="checkbox"
                                checked={checked}
                                onChange={() => {
                                  setEditMapping((prev) => {
                                    const current = prev[pv] ?? [];
                                    const next = checked
                                      ? current.filter((v) => v !== cv)
                                      : [...current, cv];
                                    return { ...prev, [pv]: next };
                                  });
                                }}
                              />
                              {cv}
                            </label>
                          );
                        })}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        );
      })()}
    </div>
  );
}
