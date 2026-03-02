import { useCallback, useEffect, useState } from "react";
import {
  api,
  isReviewed,
  type AttributeDefinition,
  type ObjectType,
  type ReqObject,
} from "./api/client";
import Markdown from "react-markdown";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  objectId: string;
  attrDefs: AttributeDefinition[];
  objectTypes: ObjectType[];
  inline?: boolean;
  onClose: () => void;
  onSaved: () => void;
}

export function ObjectDetailPanel({
  moduleId,
  objectId,
  attrDefs,
  objectTypes,
  inline,
  onClose,
  onSaved,
}: Props) {
  const [obj, setObj] = useState<ReqObject | null>(null);
  const [heading, setHeading] = useState("");
  const [body, setBody] = useState("");
  const [classification, setClassification] = useState("normative");
  const [objectTypeId, setObjectTypeId] = useState("");
  const [attributes, setAttributes] = useState<Record<string, unknown>>({});
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [previewMode, setPreviewMode] = useState<"edit" | "preview" | "split">("split");

  const fetchObject = useCallback(async () => {
    try {
      const data = await api.getObject(moduleId, objectId);
      setObj(data);
      setHeading(data.heading ?? "");
      setBody(data.body ?? "");
      setClassification(data.classification);
      setObjectTypeId(data.object_type_id ?? "");
      setAttributes((data.attributes as Record<string, unknown>) ?? {});
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load object");
    }
  }, [moduleId, objectId]);

  useEffect(() => {
    fetchObject();
  }, [fetchObject]);

  const handleSave = async () => {
    setSaving(true);
    try {
      await api.updateObject(moduleId, objectId, {
        heading: heading || undefined,
        body: body || undefined,
        classification,
        object_type_id: objectTypeId || null,
        attributes,
      });
      setError(null);
      onSaved();
      await fetchObject();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save object");
    } finally {
      setSaving(false);
    }
  };

  const handleAttrChange = (name: string, value: string) => {
    setAttributes((prev) => ({ ...prev, [name]: value }));
  };

  const content = (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: theme.spacing.sm,
        }}
      >
        <h3 style={{ margin: 0 }}>
          Object: {obj?.level ?? ""} {obj?.heading ?? objectId.slice(0, 8) + "..."}
        </h3>
        <div style={{ display: "flex", gap: theme.spacing.sm }}>
          <button onClick={handleSave} disabled={saving}>
            {saving ? "Saving..." : "Save"}
          </button>
          <button onClick={onClose}>Close</button>
        </div>
      </div>

      {error && (
        <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>
          {error}
        </div>
      )}

      {obj && (
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: theme.spacing.md }}>
          {/* Left column: Heading, Body, Classification */}
          <div>
            <label style={{ display: "block", fontWeight: 600, fontSize: "0.85rem", marginBottom: "4px" }}>
              Heading
            </label>
            <input
              type="text"
              value={heading}
              onChange={(e) => setHeading(e.target.value)}
              style={{ width: "100%", padding: theme.spacing.sm, marginBottom: theme.spacing.sm, boxSizing: "border-box" }}
            />

            <label style={{ display: "block", fontWeight: 600, fontSize: "0.85rem", marginBottom: "4px" }}>
              Body
              <span style={{ display: "inline-flex", gap: 2, marginLeft: theme.spacing.sm, fontWeight: 400 }}>
                {(["edit", "split", "preview"] as const).map((m) => (
                  <button
                    key={m}
                    onClick={() => setPreviewMode(m)}
                    style={{
                      padding: "1px 6px",
                      fontSize: "0.75rem",
                      background: previewMode === m ? theme.colors.primary : "transparent",
                      color: previewMode === m ? "#fff" : theme.colors.text,
                      border: `1px solid ${theme.colors.border}`,
                      borderRadius: 3,
                      cursor: "pointer",
                    }}
                  >
                    {m.charAt(0).toUpperCase() + m.slice(1)}
                  </button>
                ))}
              </span>
            </label>
            <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: theme.spacing.sm }}>
              {(previewMode === "edit" || previewMode === "split") && (
                <textarea
                  value={body}
                  onChange={(e) => setBody(e.target.value)}
                  rows={10}
                  style={{
                    flex: 1,
                    padding: theme.spacing.sm,
                    resize: "vertical",
                    fontFamily: "monospace",
                    fontSize: "0.85rem",
                    boxSizing: "border-box",
                  }}
                />
              )}
              {(previewMode === "preview" || previewMode === "split") && (
                <div
                  style={{
                    flex: 1,
                    padding: theme.spacing.sm,
                    border: `1px solid ${theme.colors.borderLight}`,
                    borderRadius: theme.borderRadius,
                    overflow: "auto",
                    minHeight: 200,
                    fontSize: "0.9rem",
                  }}
                >
                  <Markdown>{body}</Markdown>
                </div>
              )}
            </div>

            <label style={{ display: "block", fontWeight: 600, fontSize: "0.85rem", marginBottom: "4px" }}>
              Classification
            </label>
            <select
              value={classification}
              onChange={(e) => setClassification(e.target.value)}
              style={{ padding: theme.spacing.sm, marginBottom: theme.spacing.sm }}
            >
              <option value="normative">Normative</option>
              <option value="informative">Informative</option>
              <option value="heading">Heading</option>
            </select>

            {objectTypes.length > 0 && (
              <>
                <label style={{ display: "block", fontWeight: 600, fontSize: "0.85rem", marginBottom: "4px" }}>
                  Type
                </label>
                <select
                  value={objectTypeId}
                  onChange={(e) => setObjectTypeId(e.target.value)}
                  style={{ padding: theme.spacing.sm, marginBottom: theme.spacing.sm }}
                >
                  <option value="">(none)</option>
                  {objectTypes.map((t) => (
                    <option key={t.id} value={t.id}>{t.name}</option>
                  ))}
                </select>
              </>
            )}
          </div>

          {/* Right column: Attributes, Meta */}
          <div>
            {attrDefs.length > 0 && (
              <>
                <label style={{ display: "block", fontWeight: 600, fontSize: "0.85rem", marginBottom: "4px" }}>
                  Attributes
                </label>
                {attrDefs.map((def) => (
                  <div key={def.id} style={{ marginBottom: theme.spacing.sm }}>
                    <label style={{ fontSize: "0.8rem", color: theme.colors.textSecondary }}>
                      {def.name}
                    </label>
                    {def.data_type === "enum" && Array.isArray(def.enum_values) ? (
                      <select
                        value={String(attributes[def.name] ?? "")}
                        onChange={(e) => handleAttrChange(def.name, e.target.value)}
                        style={{ display: "block", padding: "4px", width: "100%" }}
                      >
                        <option value="">(none)</option>
                        {def.enum_values.map((v) => (
                          <option key={v} value={v}>{v}</option>
                        ))}
                      </select>
                    ) : (
                      <input
                        type="text"
                        value={String(attributes[def.name] ?? "")}
                        onChange={(e) => handleAttrChange(def.name, e.target.value)}
                        style={{ display: "block", padding: "4px", width: "100%", boxSizing: "border-box" }}
                      />
                    )}
                  </div>
                ))}
              </>
            )}

            <div style={{ marginTop: theme.spacing.md, fontSize: "0.8rem", color: theme.colors.textMuted }}>
              <div>Version: {obj.current_version}</div>
              <div>Reviewed: {isReviewed(obj) ? "Yes" : "No"}</div>
              {obj.reviewed_at && <div>Reviewed at: {new Date(obj.reviewed_at).toLocaleString()}</div>}
              <div>Created: {new Date(obj.created_at).toLocaleString()}</div>
              <div>Updated: {new Date(obj.updated_at).toLocaleString()}</div>
              <div>ID: {obj.id}</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );

  if (inline) {
    return <div style={{ padding: theme.spacing.md }}>{content}</div>;
  }

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
          maxWidth: 900,
          width: "90%",
          maxHeight: "85vh",
          overflow: "auto",
          boxShadow: "0 8px 32px rgba(0,0,0,0.2)",
        }}
      >
        {content}
      </div>
    </div>
  );
}
