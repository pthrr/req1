import { useState } from "react";
import Markdown from "react-markdown";
import { api, type ReqObject } from "./api/client";
import { RichTextEditor } from "./RichTextEditor";
import { isHtmlContent, prepareBodyForEditor } from "./utils/bodyFormat";
import { theme } from "./theme";

interface DocBlockProps {
  moduleId: string;
  object: ReqObject;
  isEditing: boolean;
  onStartEdit: () => void;
  onStopEdit: () => void;
  onSaved: () => void;
}

export function DocBlock({
  moduleId,
  object,
  isEditing,
  onStartEdit,
  onStopEdit,
  onSaved,
}: DocBlockProps) {
  const [heading, setHeading] = useState(object.heading ?? "");
  const [body, setBody] = useState(object.body ?? "");
  const [editorBody, setEditorBody] = useState(prepareBodyForEditor(object.body));
  const [classification, setClassification] = useState(object.classification);
  const [saving, setSaving] = useState(false);

  const depth = object.level.split(".").length - 1;
  const headingLevel = Math.min(depth + 2, 6);

  const borderColor =
    object.classification === "informative"
      ? "#1565c0"
      : object.classification === "heading"
        ? "#6a1b9a"
        : "transparent";

  const handleSave = async () => {
    setSaving(true);
    try {
      await api.updateObject(moduleId, object.id, {
        heading: heading || undefined,
        body: body || undefined,
        classification,
      });
      onSaved();
      onStopEdit();
    } catch {
      // Keep editing on error
    } finally {
      setSaving(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") onStopEdit();
  };

  if (isEditing) {
    return (
      <div
        onKeyDown={handleKeyDown}
        style={{
          padding: theme.spacing.md,
          borderLeft: `3px solid ${theme.colors.primary}`,
          background: "#f8f9ff",
          marginBottom: theme.spacing.sm,
          borderRadius: theme.borderRadius,
        }}
      >
        <div style={{ marginBottom: theme.spacing.sm }}>
          <label style={{ fontSize: "0.8rem", fontWeight: 600 }}>Heading</label>
          <input
            type="text"
            value={heading}
            onChange={(e) => setHeading(e.target.value)}
            style={{ width: "100%", padding: theme.spacing.sm, boxSizing: "border-box" }}
          />
        </div>
        <div style={{ marginBottom: theme.spacing.sm }}>
          <label style={{ fontSize: "0.8rem", fontWeight: 600 }}>Body</label>
          <RichTextEditor
            content={editorBody}
            onChange={(html) => { setEditorBody(html); setBody(html); }}
            objectId={object.id}
            minHeight={120}
          />
        </div>
        <div style={{ marginBottom: theme.spacing.sm }}>
          <label style={{ fontSize: "0.8rem", fontWeight: 600 }}>Classification</label>
          <select
            value={classification}
            onChange={(e) => setClassification(e.target.value)}
            style={{ padding: theme.spacing.sm, display: "block" }}
          >
            <option value="normative">Normative</option>
            <option value="informative">Informative</option>
            <option value="heading">Heading</option>
          </select>
        </div>
        <div style={{ display: "flex", gap: theme.spacing.sm }}>
          <button onClick={handleSave} disabled={saving}>
            {saving ? "Saving..." : "Save"}
          </button>
          <button onClick={onStopEdit}>Cancel</button>
        </div>
      </div>
    );
  }

  return (
    <div
      onDoubleClick={onStartEdit}
      style={{
        padding: `${theme.spacing.sm} ${theme.spacing.md}`,
        borderLeft: `3px solid ${borderColor}`,
        marginBottom: theme.spacing.xs,
        cursor: "default",
        borderRadius: theme.borderRadius,
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.background = theme.colors.bgHover;
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = "transparent";
      }}
    >
      {object.heading && (
        <div
          role="heading"
          aria-level={headingLevel}
          style={{
            margin: "0 0 4px 0",
            color: "#1565c0",
            fontSize: `${1.4 - (headingLevel - 2) * 0.15}rem`,
            fontWeight: 600,
          }}
        >
          <span style={{ color: theme.colors.textMuted, fontWeight: 400, marginRight: 6, fontSize: "0.85em" }}>
            {object.level}
          </span>
          {object.heading}
        </div>
      )}
      {object.body && (
        <div style={{ fontSize: "0.9rem", lineHeight: 1.6 }}>
          {isHtmlContent(object.body) ? (
            <div dangerouslySetInnerHTML={{ __html: object.body }} />
          ) : (
            <Markdown>{object.body}</Markdown>
          )}
        </div>
      )}
      {!object.heading && !object.body && (
        <div style={{ color: theme.colors.textMuted, fontStyle: "italic", fontSize: "0.85rem" }}>
          (empty object — double-click to edit)
        </div>
      )}
    </div>
  );
}
