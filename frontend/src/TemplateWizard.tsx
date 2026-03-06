import { useEffect, useState } from "react";
import { api, type ProjectTemplate } from "./api/client";
import { theme } from "./theme";

interface Props {
  workspaceId: string;
  onClose: () => void;
  onCreated: () => void;
}

type Step = "select" | "configure" | "review";

export function TemplateWizard({ workspaceId, onClose, onCreated }: Props) {
  const [step, setStep] = useState<Step>("select");
  const [templates, setTemplates] = useState<ProjectTemplate[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [projectName, setProjectName] = useState("");
  const [projectDescription, setProjectDescription] = useState("");
  const [includeSeedObjects, setIncludeSeedObjects] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<{ project_id: string; modules_created: number } | null>(null);

  useEffect(() => {
    api.listProjectTemplates().then(setTemplates).catch(() => {});
  }, []);

  const selected = templates.find((t) => t.id === selectedId);

  const moduleCount = (() => {
    if (!selected) return 0;
    const data = selected.template_data as { modules?: unknown[] } | null;
    return data?.modules?.length ?? 0;
  })();

  const handleCreate = async () => {
    if (!selectedId || !projectName.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.instantiateTemplate(selectedId, {
        workspace_id: workspaceId,
        project_name: projectName.trim(),
        project_description: projectDescription.trim() || undefined,
        include_seed_objects: includeSeedObjects,
      });
      setResult(res);
      onCreated();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create project");
    } finally {
      setLoading(false);
    }
  };

  const overlayStyle: React.CSSProperties = {
    position: "fixed",
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    background: "rgba(0,0,0,0.4)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    zIndex: 1000,
  };

  const modalStyle: React.CSSProperties = {
    background: theme.colors.bg,
    borderRadius: theme.borderRadius,
    padding: theme.spacing.lg,
    minWidth: 500,
    maxWidth: 700,
    maxHeight: "80vh",
    overflowY: "auto",
    boxShadow: "0 8px 32px rgba(0,0,0,0.2)",
  };

  const cardStyle = (isSelected: boolean): React.CSSProperties => ({
    border: `2px solid ${isSelected ? theme.colors.primary : theme.colors.border}`,
    borderRadius: theme.borderRadius,
    padding: theme.spacing.md,
    cursor: "pointer",
    background: isSelected ? "#e3f2fd" : "transparent",
  });

  return (
    <div style={overlayStyle} onClick={onClose}>
      <div style={modalStyle} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: theme.spacing.md }}>
          <h2 style={{ margin: 0 }}>Create Project from Template</h2>
          <button onClick={onClose} style={{ background: "none", border: "none", fontSize: "1.2rem", cursor: "pointer" }}>
            &#10005;
          </button>
        </div>

        {/* Step indicators */}
        <div style={{ display: "flex", gap: theme.spacing.md, marginBottom: theme.spacing.lg }}>
          {(["select", "configure", "review"] as Step[]).map((s, i) => (
            <div
              key={s}
              style={{
                flex: 1,
                padding: theme.spacing.sm,
                textAlign: "center",
                borderBottom: `2px solid ${step === s ? theme.colors.primary : theme.colors.borderLight}`,
                color: step === s ? theme.colors.primary : theme.colors.textMuted,
                fontWeight: step === s ? 600 : 400,
                fontSize: "0.9rem",
              }}
            >
              {i + 1}. {s.charAt(0).toUpperCase() + s.slice(1)}
            </div>
          ))}
        </div>

        {error && (
          <div style={{ color: theme.colors.error, marginBottom: theme.spacing.md }}>{error}</div>
        )}

        {/* Step 1: Select template */}
        {step === "select" && (
          <div>
            <p style={{ fontSize: "0.9rem", marginBottom: theme.spacing.md }}>
              Choose a compliance template to start from.
            </p>
            <div style={{ display: "flex", flexDirection: "column", gap: theme.spacing.sm }}>
              {templates.map((t) => (
                <div
                  key={t.id}
                  style={cardStyle(selectedId === t.id)}
                  onClick={() => setSelectedId(t.id)}
                >
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                    <strong>{t.name}</strong>
                    {t.is_builtin && (
                      <span
                        style={{
                          fontSize: "0.7rem",
                          padding: "2px 6px",
                          background: theme.colors.primary,
                          color: "#fff",
                          borderRadius: "10px",
                        }}
                      >
                        Built-in
                      </span>
                    )}
                  </div>
                  {t.standard && (
                    <div style={{ fontSize: "0.85rem", color: theme.colors.textSecondary, marginTop: 2 }}>
                      Standard: {t.standard}
                      {t.version && ` v${t.version}`}
                    </div>
                  )}
                  {t.description && (
                    <div style={{ fontSize: "0.85rem", marginTop: 4 }}>{t.description}</div>
                  )}
                </div>
              ))}
              {templates.length === 0 && (
                <p style={{ color: theme.colors.textMuted }}>No templates available.</p>
              )}
            </div>
            <div style={{ display: "flex", gap: theme.spacing.sm, justifyContent: "flex-end", marginTop: theme.spacing.md }}>
              <button onClick={onClose} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
                Cancel
              </button>
              <button
                onClick={() => setStep("configure")}
                disabled={!selectedId}
                style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
              >
                Next
              </button>
            </div>
          </div>
        )}

        {/* Step 2: Configure */}
        {step === "configure" && (
          <div>
            <div style={{ marginBottom: theme.spacing.md }}>
              <label style={{ display: "block", marginBottom: 4, fontWeight: 600, fontSize: "0.9rem" }}>
                Project Name *
              </label>
              <input
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                placeholder="e.g., Vehicle ECU Safety Case"
                style={{ width: "100%", padding: theme.spacing.sm, boxSizing: "border-box" }}
              />
            </div>
            <div style={{ marginBottom: theme.spacing.md }}>
              <label style={{ display: "block", marginBottom: 4, fontWeight: 600, fontSize: "0.9rem" }}>
                Description
              </label>
              <textarea
                value={projectDescription}
                onChange={(e) => setProjectDescription(e.target.value)}
                placeholder="Optional description..."
                rows={3}
                style={{ width: "100%", padding: theme.spacing.sm, boxSizing: "border-box", resize: "vertical" }}
              />
            </div>
            <label style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "0.9rem", cursor: "pointer" }}>
              <input
                type="checkbox"
                checked={includeSeedObjects}
                onChange={(e) => setIncludeSeedObjects(e.target.checked)}
              />
              Include seed objects (pre-populated headings and structure)
            </label>
            <div style={{ display: "flex", gap: theme.spacing.sm, justifyContent: "flex-end", marginTop: theme.spacing.lg }}>
              <button onClick={() => setStep("select")} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
                Back
              </button>
              <button
                onClick={() => setStep("review")}
                disabled={!projectName.trim()}
                style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
              >
                Next
              </button>
            </div>
          </div>
        )}

        {/* Step 3: Review & Create */}
        {step === "review" && !result && (
          <div>
            <h3 style={{ marginTop: 0 }}>Review</h3>
            <table style={{ width: "100%", borderCollapse: "collapse", marginBottom: theme.spacing.md }}>
              <tbody>
                <tr>
                  <td style={{ padding: "6px", fontWeight: 600 }}>Template</td>
                  <td style={{ padding: "6px" }}>{selected?.name}</td>
                </tr>
                <tr>
                  <td style={{ padding: "6px", fontWeight: 600 }}>Standard</td>
                  <td style={{ padding: "6px" }}>{selected?.standard ?? "-"}</td>
                </tr>
                <tr>
                  <td style={{ padding: "6px", fontWeight: 600 }}>Project Name</td>
                  <td style={{ padding: "6px" }}>{projectName}</td>
                </tr>
                <tr>
                  <td style={{ padding: "6px", fontWeight: 600 }}>Modules</td>
                  <td style={{ padding: "6px" }}>{moduleCount}</td>
                </tr>
                <tr>
                  <td style={{ padding: "6px", fontWeight: 600 }}>Seed Objects</td>
                  <td style={{ padding: "6px" }}>{includeSeedObjects ? "Yes" : "No"}</td>
                </tr>
              </tbody>
            </table>
            <div style={{ display: "flex", gap: theme.spacing.sm, justifyContent: "flex-end" }}>
              <button onClick={() => setStep("configure")} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
                Back
              </button>
              <button
                onClick={handleCreate}
                disabled={loading}
                style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, fontWeight: 600 }}
              >
                {loading ? "Creating..." : "Create Project"}
              </button>
            </div>
          </div>
        )}

        {/* Result */}
        {result && (
          <div>
            <h3>Project Created</h3>
            <p>Modules created: {result.modules_created}</p>
            <div style={{ display: "flex", justifyContent: "flex-end", marginTop: theme.spacing.md }}>
              <button onClick={onClose} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
                Close
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
