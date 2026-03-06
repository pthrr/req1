import { useEffect, useState } from "react";
import { api } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  onClose: () => void;
}

export function TemplateEditorPanel({ moduleId, onClose }: Props) {
  const [template, setTemplate] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);

  // Load existing template from module on mount
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const mod = await api.getModule(moduleId);
        if (!cancelled && mod.publish_template) {
          setTemplate(mod.publish_template);
        }
      } catch {
        // Non-critical — template field may be empty
      }
    })();
    return () => { cancelled = true; };
  }, [moduleId]);

  const handleSave = async () => {
    try {
      await api.updateModule(moduleId, { publish_template: template });
      setSaved(true);
      setError(null);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save template");
    }
  };

  const handlePreview = () => {
    setPreviewUrl(api.getPublishUrl(moduleId, "html"));
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
          maxWidth: 900,
          width: "90%",
          height: "80vh",
          display: "flex",
          flexDirection: "column",
          boxShadow: "0 8px 32px rgba(0,0,0,0.2)",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: theme.spacing.sm,
            flexShrink: 0,
          }}
        >
          <h3 style={{ margin: 0 }}>Publish Template Editor</h3>
          <div style={{ display: "flex", gap: theme.spacing.sm }}>
            <button onClick={handlePreview}>Preview</button>
            <button onClick={handleSave}>
              {saved ? "Saved!" : "Save"}
            </button>
            <button onClick={onClose}>Close</button>
          </div>
        </div>

        {error && (
          <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm, flexShrink: 0 }}>
            {error}
          </div>
        )}

        <p style={{ fontSize: "0.85rem", color: theme.colors.textMuted, margin: `0 0 ${theme.spacing.sm}`, flexShrink: 0 }}>
          Write a Minijinja template. Available variables: objects, module, baselines, links.
        </p>

        <div style={{ display: "flex", flex: 1, gap: theme.spacing.sm, minHeight: 0 }}>
          <textarea
            value={template}
            onChange={(e) => setTemplate(e.target.value)}
            placeholder={`<!DOCTYPE html>\n<html>\n<head><title>{{ module.name }}</title></head>\n<body>\n{% for obj in objects %}\n  <div>\n    <h{{ obj.level | length }}>{{ obj.heading }}</h{{ obj.level | length }}>\n    <p>{{ obj.body }}</p>\n  </div>\n{% endfor %}\n</body>\n</html>`}
            style={{
              flex: 1,
              padding: theme.spacing.sm,
              fontFamily: "monospace",
              fontSize: "0.85rem",
              resize: "none",
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
            }}
          />
          {previewUrl && (
            <iframe
              src={previewUrl}
              style={{
                flex: 1,
                border: `1px solid ${theme.colors.borderLight}`,
                borderRadius: theme.borderRadius,
              }}
              title="Template Preview"
            />
          )}
        </div>
      </div>
    </div>
  );
}
