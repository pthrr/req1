import { useState } from "react";
import {
  api,
  type AttributeDefinition,
  type FormLayout,
  type FormSection,
  type ObjectType,
} from "./api/client";
import { theme } from "./theme";

interface Props {
  objectType: ObjectType;
  attrDefs: AttributeDefinition[];
  onSave: () => void;
  onClose: () => void;
}

function makeEmptySection(): FormSection {
  return {
    id: `section_${Date.now()}`,
    title: "New Section",
    columns: 1,
    fields: [],
  };
}

export function FormLayoutEditor({ objectType, attrDefs, onSave, onClose }: Props) {
  const existing = objectType.attribute_schema as FormLayout | Record<string, never>;
  const initialSections: FormSection[] =
    existing && "sections" in existing && Array.isArray(existing.sections)
      ? existing.sections
      : [];

  const [sections, setSections] = useState<FormSection[]>(initialSections);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  // Collect all attribute names currently placed in sections
  const usedNames = new Set(
    sections.flatMap((s) => s.fields.map((f) => f.attribute_name)),
  );
  const availableAttrs = attrDefs.filter((d) => !usedNames.has(d.name));

  const handleSave = async () => {
    setSaving(true);
    try {
      const schema: FormLayout = { sections };
      await api.updateObjectType(objectType.id, {
        attribute_schema: sections.length > 0 ? schema : {},
      });
      setError(null);
      onSave();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save layout");
    } finally {
      setSaving(false);
    }
  };

  const addSection = () => {
    setSections((prev) => [...prev, makeEmptySection()]);
  };

  const removeSection = (index: number) => {
    setSections((prev) => prev.filter((_, i) => i !== index));
  };

  const updateSection = (index: number, patch: Partial<FormSection>) => {
    setSections((prev) =>
      prev.map((s, i) => (i === index ? { ...s, ...patch } : s)),
    );
  };

  const addField = (sectionIndex: number, attrName: string) => {
    setSections((prev) =>
      prev.map((s, i) => {
        if (i !== sectionIndex) return s;
        return {
          ...s,
          fields: [
            ...s.fields,
            {
              attribute_name: attrName,
              order: s.fields.length,
              width: undefined,
              required: undefined,
            },
          ],
        };
      }),
    );
  };

  const removeField = (sectionIndex: number, fieldIndex: number) => {
    setSections((prev) =>
      prev.map((s, i) => {
        if (i !== sectionIndex) return s;
        return {
          ...s,
          fields: s.fields.filter((_, fi) => fi !== fieldIndex),
        };
      }),
    );
  };

  const moveField = (
    sectionIndex: number,
    fieldIndex: number,
    direction: "up" | "down",
  ) => {
    setSections((prev) =>
      prev.map((s, i) => {
        if (i !== sectionIndex) return s;
        const fields = [...s.fields];
        const swapIndex =
          direction === "up" ? fieldIndex - 1 : fieldIndex + 1;
        if (swapIndex < 0 || swapIndex >= fields.length) return s;
        [fields[fieldIndex], fields[swapIndex]] = [
          fields[swapIndex],
          fields[fieldIndex],
        ];
        return { ...s, fields };
      }),
    );
  };

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: theme.colors.overlayBg,
        zIndex: 1000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        style={{
          background: theme.colors.bg,
          borderRadius: 8,
          padding: theme.spacing.lg,
          maxWidth: 800,
          width: "90%",
          maxHeight: "85vh",
          overflow: "auto",
          boxShadow: "0 8px 32px rgba(0,0,0,0.2)",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: theme.spacing.md,
          }}
        >
          <h3 style={{ margin: 0 }}>
            Form Layout: {objectType.name}
          </h3>
          <div style={{ display: "flex", gap: theme.spacing.sm }}>
            <button onClick={handleSave} disabled={saving}>
              {saving ? "Saving..." : "Save"}
            </button>
            <button onClick={onClose}>Close</button>
          </div>
        </div>

        {error && (
          <div
            style={{
              color: theme.colors.error,
              marginBottom: theme.spacing.sm,
            }}
          >
            {error}
          </div>
        )}

        <div style={{ display: "flex", gap: theme.spacing.lg }}>
          {/* Left: available attributes */}
          <div style={{ width: 200, flexShrink: 0 }}>
            <h4 style={{ margin: `0 0 ${theme.spacing.sm}`, fontSize: "0.9rem" }}>
              Available Attributes
            </h4>
            {availableAttrs.length === 0 ? (
              <div
                style={{
                  color: theme.colors.textMuted,
                  fontSize: "0.85rem",
                }}
              >
                All attributes are placed
              </div>
            ) : (
              availableAttrs.map((d) => (
                <div
                  key={d.id}
                  style={{
                    padding: "4px 8px",
                    marginBottom: 4,
                    background: theme.colors.bgCode,
                    borderRadius: theme.borderRadius,
                    fontSize: "0.85rem",
                    cursor: "grab",
                  }}
                >
                  {d.name}
                  <span
                    style={{
                      marginLeft: 4,
                      color: theme.colors.textMuted,
                      fontSize: "0.75rem",
                    }}
                  >
                    ({d.data_type})
                  </span>
                </div>
              ))
            )}
          </div>

          {/* Right: sections */}
          <div style={{ flex: 1 }}>
            {sections.map((section, si) => (
              <div
                key={section.id}
                style={{
                  border: `1px solid ${theme.colors.borderLight}`,
                  borderRadius: theme.borderRadius,
                  padding: theme.spacing.sm,
                  marginBottom: theme.spacing.sm,
                }}
              >
                <div
                  style={{
                    display: "flex",
                    gap: theme.spacing.sm,
                    alignItems: "center",
                    marginBottom: theme.spacing.sm,
                  }}
                >
                  <input
                    type="text"
                    value={section.title}
                    onChange={(e) =>
                      updateSection(si, { title: e.target.value })
                    }
                    style={{ flex: 1, padding: "4px", fontWeight: 600 }}
                  />
                  <label
                    style={{
                      fontSize: "0.8rem",
                      display: "flex",
                      alignItems: "center",
                      gap: 4,
                    }}
                  >
                    Cols:
                    <select
                      value={section.columns}
                      onChange={(e) =>
                        updateSection(si, {
                          columns: Number(e.target.value) as 1 | 2,
                        })
                      }
                      style={{ padding: "2px" }}
                    >
                      <option value={1}>1</option>
                      <option value={2}>2</option>
                    </select>
                  </label>
                  <button
                    onClick={() => removeSection(si)}
                    style={{
                      padding: "2px 6px",
                      fontSize: "0.8rem",
                      color: theme.colors.error,
                    }}
                  >
                    Remove
                  </button>
                </div>

                {/* Fields */}
                {section.fields.map((field, fi) => (
                  <div
                    key={field.attribute_name}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: theme.spacing.xs,
                      padding: "2px 4px",
                      marginBottom: 2,
                      background: theme.colors.bgCode,
                      borderRadius: 3,
                      fontSize: "0.85rem",
                    }}
                  >
                    <span style={{ flex: 1 }}>{field.attribute_name}</span>
                    <button
                      onClick={() => moveField(si, fi, "up")}
                      disabled={fi === 0}
                      style={{ padding: "1px 4px", fontSize: "0.75rem" }}
                    >
                      Up
                    </button>
                    <button
                      onClick={() => moveField(si, fi, "down")}
                      disabled={fi === section.fields.length - 1}
                      style={{ padding: "1px 4px", fontSize: "0.75rem" }}
                    >
                      Dn
                    </button>
                    <button
                      onClick={() => removeField(si, fi)}
                      style={{ padding: "1px 4px", fontSize: "0.75rem" }}
                    >
                      X
                    </button>
                  </div>
                ))}

                {/* Add field dropdown */}
                {availableAttrs.length > 0 && (
                  <select
                    onChange={(e) => {
                      if (e.target.value) {
                        addField(si, e.target.value);
                        e.target.value = "";
                      }
                    }}
                    style={{
                      marginTop: theme.spacing.xs,
                      padding: "4px",
                      fontSize: "0.8rem",
                    }}
                  >
                    <option value="">+ Add field...</option>
                    {availableAttrs.map((d) => (
                      <option key={d.id} value={d.name}>
                        {d.name}
                      </option>
                    ))}
                  </select>
                )}
              </div>
            ))}

            <button
              onClick={addSection}
              style={{
                padding: `${theme.spacing.sm} ${theme.spacing.md}`,
                fontSize: "0.85rem",
              }}
            >
              + Add Section
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
