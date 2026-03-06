import { useEffect, useRef, useState } from "react";
import { api } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  onClose: () => void;
}

const PREVIEW_FORMATS = [
  { value: "html", label: "HTML" },
  { value: "markdown", label: "Markdown" },
  { value: "latex", label: "LaTeX" },
  { value: "txt", label: "Plain Text" },
  { value: "csv", label: "CSV" },
  { value: "yaml", label: "YAML" },
  { value: "pdf", label: "PDF" },
  { value: "xlsx", label: "Excel (XLSX)" },
  { value: "docx", label: "Word (DOCX)" },
];

export function PublishPreviewPanel({ moduleId, onClose }: Props) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [format, setFormat] = useState("html");
  const [textContent, setTextContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const isHtml = format === "html";
  const isPdf = format === "pdf";
  const isXlsx = format === "xlsx";
  const isDocx = format === "docx";
  const isBinary = isPdf || isXlsx || isDocx;
  const isIframe = isHtml || isPdf;

  useEffect(() => {
    if (isHtml || isBinary) {
      setTextContent(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    fetch(api.getPublishUrl(moduleId, format))
      .then((res) => {
        if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
        return res.text();
      })
      .then((text) => {
        if (!cancelled) setTextContent(text);
      })
      .catch((err) => {
        if (!cancelled) setTextContent(`Error: ${err.message}`);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [moduleId, format, isHtml, isBinary]);

  const handlePrint = () => {
    iframeRef.current?.contentWindow?.print();
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
          maxWidth: 1000,
          width: "95%",
          height: "90vh",
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
          <div style={{ display: "flex", alignItems: "center", gap: theme.spacing.sm }}>
            <h3 style={{ margin: 0 }}>Document Preview</h3>
            <select
              value={format}
              onChange={(e) => setFormat(e.target.value)}
              style={{ padding: "4px 8px", fontSize: "0.9rem" }}
            >
              {PREVIEW_FORMATS.map((f) => (
                <option key={f.value} value={f.value}>
                  {f.label}
                </option>
              ))}
            </select>
          </div>
          <div style={{ display: "flex", gap: theme.spacing.sm }}>
            {isHtml && <button onClick={handlePrint}>Print</button>}
            {isXlsx && (
              <a
                href={api.getPublishUrl(moduleId, "xlsx")}
                download="objects.xlsx"
                style={{ textDecoration: "none" }}
              >
                <button type="button">Download</button>
              </a>
            )}
            {isDocx && (
              <a
                href={api.getPublishUrl(moduleId, "docx")}
                download="document.docx"
                style={{ textDecoration: "none" }}
              >
                <button type="button">Download</button>
              </a>
            )}
            <button onClick={onClose}>Close</button>
          </div>
        </div>
        {isIframe ? (
          <iframe
            ref={iframeRef}
            src={api.getPublishUrl(moduleId, format)}
            style={{
              flex: 1,
              border: `1px solid ${theme.colors.borderLight}`,
              borderRadius: theme.borderRadius,
            }}
            title="Document Preview"
          />
        ) : (isXlsx || isDocx) ? (
          <div
            style={{
              flex: 1,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              border: `1px solid ${theme.colors.borderLight}`,
              borderRadius: theme.borderRadius,
              color: theme.colors.textMuted,
            }}
          >
            {isXlsx ? "Excel" : "Word"} files cannot be previewed. Use the Download button above.
          </div>
        ) : (
          <pre
            style={{
              flex: 1,
              overflow: "auto",
              border: `1px solid ${theme.colors.borderLight}`,
              borderRadius: theme.borderRadius,
              padding: theme.spacing.md,
              margin: 0,
              fontSize: "0.85rem",
              whiteSpace: "pre-wrap",
              wordWrap: "break-word",
              background: theme.colors.bgCode ?? "#f8f9fa",
            }}
          >
            {loading ? "Loading..." : textContent}
          </pre>
        )}
      </div>
    </div>
  );
}
