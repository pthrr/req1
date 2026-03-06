import { useState, useRef } from "react";
import { api, type DocxPreviewResult } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  onClose: () => void;
  onImported: () => void;
}

type Step = "upload" | "mapping" | "confirm";

interface StyleMapping {
  style_id: string;
  classification: string;
  is_heading: boolean;
}

export function DocxImportWizard({ moduleId, onClose, onImported }: Props) {
  const [step, setStep] = useState<Step>("upload");
  const [file, setFile] = useState<File | null>(null);
  const [preview, setPreview] = useState<DocxPreviewResult | null>(null);
  const [mappings, setMappings] = useState<StyleMapping[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<{ objects_created: number; objects_updated: number; paragraphs_skipped: number } | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const handleUpload = async () => {
    if (!file) return;
    setLoading(true);
    setError(null);
    try {
      const previewData = await api.previewDocx(moduleId, file);
      setPreview(previewData);
      setMappings(
        previewData.styles.map((s) => ({
          style_id: s.style_id,
          classification: s.style_id.toLowerCase().includes("heading") ? "heading" : "normative",
          is_heading: s.style_id.toLowerCase().includes("heading") || s.style_id === "Title",
        })),
      );
      setStep("mapping");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Preview failed");
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async () => {
    if (!file) return;
    setLoading(true);
    setError(null);
    try {
      const importResult = await api.importDocx(moduleId, file, mappings);
      setResult(importResult);
      onImported();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Import failed");
    } finally {
      setLoading(false);
    }
  };

  const updateMapping = (idx: number, field: keyof StyleMapping, value: string | boolean) => {
    setMappings((prev) => {
      const next = [...prev];
      next[idx] = { ...next[idx], [field]: value };
      return next;
    });
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

  return (
    <div style={overlayStyle} onClick={onClose}>
      <div style={modalStyle} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: theme.spacing.md }}>
          <h2 style={{ margin: 0 }}>Import DOCX</h2>
          <button onClick={onClose} style={{ background: "none", border: "none", fontSize: "1.2rem", cursor: "pointer" }}>
            &#10005;
          </button>
        </div>

        {/* Step indicators */}
        <div style={{ display: "flex", gap: theme.spacing.md, marginBottom: theme.spacing.lg }}>
          {(["upload", "mapping", "confirm"] as Step[]).map((s, i) => (
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

        {/* Step 1: Upload */}
        {step === "upload" && (
          <div>
            <p>Select a .docx file to import.</p>
            <input
              ref={fileRef}
              type="file"
              accept=".docx"
              onChange={(e) => setFile(e.target.files?.[0] ?? null)}
              style={{ marginBottom: theme.spacing.md }}
            />
            {file && (
              <p style={{ fontSize: "0.9rem", color: theme.colors.textSecondary }}>
                Selected: {file.name} ({Math.round(file.size / 1024)} KB)
              </p>
            )}
            <div style={{ display: "flex", gap: theme.spacing.sm, justifyContent: "flex-end" }}>
              <button onClick={onClose} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
                Cancel
              </button>
              <button
                onClick={handleUpload}
                disabled={!file || loading}
                style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
              >
                {loading ? "Analyzing..." : "Next"}
              </button>
            </div>
          </div>
        )}

        {/* Step 2: Mapping */}
        {step === "mapping" && preview && (
          <div>
            <p style={{ fontSize: "0.9rem", marginBottom: theme.spacing.sm }}>
              Found {preview.paragraph_count} paragraphs with {preview.styles.length} styles.
              Map each style to a classification.
            </p>
            <table style={{ width: "100%", borderCollapse: "collapse", marginBottom: theme.spacing.md }}>
              <thead>
                <tr>
                  <th style={{ textAlign: "left", padding: "6px", borderBottom: `1px solid ${theme.colors.border}` }}>Style</th>
                  <th style={{ textAlign: "left", padding: "6px", borderBottom: `1px solid ${theme.colors.border}` }}>Sample</th>
                  <th style={{ textAlign: "left", padding: "6px", borderBottom: `1px solid ${theme.colors.border}` }}>Count</th>
                  <th style={{ textAlign: "left", padding: "6px", borderBottom: `1px solid ${theme.colors.border}` }}>Classification</th>
                  <th style={{ textAlign: "center", padding: "6px", borderBottom: `1px solid ${theme.colors.border}` }}>Heading?</th>
                </tr>
              </thead>
              <tbody>
                {mappings.map((m, i) => {
                  const style = preview.styles.find((s) => s.style_id === m.style_id);
                  return (
                    <tr key={m.style_id}>
                      <td style={{ padding: "6px", fontSize: "0.85rem", fontFamily: "monospace" }}>{m.style_id}</td>
                      <td style={{ padding: "6px", fontSize: "0.85rem", maxWidth: 150, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {style?.sample_text ?? ""}
                      </td>
                      <td style={{ padding: "6px", fontSize: "0.85rem" }}>{style?.count ?? 0}</td>
                      <td style={{ padding: "6px" }}>
                        <select
                          value={m.classification}
                          onChange={(e) => updateMapping(i, "classification", e.target.value)}
                          style={{ padding: "2px 4px", fontSize: "0.85rem" }}
                        >
                          <option value="normative">Normative</option>
                          <option value="informative">Informative</option>
                          <option value="heading">Heading</option>
                        </select>
                      </td>
                      <td style={{ padding: "6px", textAlign: "center" }}>
                        <input
                          type="checkbox"
                          checked={m.is_heading}
                          onChange={(e) => updateMapping(i, "is_heading", e.target.checked)}
                        />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
            <div style={{ display: "flex", gap: theme.spacing.sm, justifyContent: "flex-end" }}>
              <button onClick={() => setStep("upload")} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
                Back
              </button>
              <button onClick={() => setStep("confirm")} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
                Next
              </button>
            </div>
          </div>
        )}

        {/* Step 3: Confirm */}
        {step === "confirm" && !result && (
          <div>
            <h3>Summary</h3>
            <p>File: {file?.name}</p>
            <p>Paragraphs: {preview?.paragraph_count}</p>
            <p>Heading styles: {mappings.filter((m) => m.is_heading).length}</p>
            <p>Body styles: {mappings.filter((m) => !m.is_heading).length}</p>
            <div style={{ display: "flex", gap: theme.spacing.sm, justifyContent: "flex-end", marginTop: theme.spacing.md }}>
              <button onClick={() => setStep("mapping")} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
                Back
              </button>
              <button
                onClick={handleImport}
                disabled={loading}
                style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, fontWeight: 600 }}
              >
                {loading ? "Importing..." : "Import"}
              </button>
            </div>
          </div>
        )}

        {/* Result */}
        {result && (
          <div>
            <h3>Import Complete</h3>
            <p>Objects created: {result.objects_created}</p>
            <p>Objects updated: {result.objects_updated}</p>
            <p>Paragraphs skipped: {result.paragraphs_skipped}</p>
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
