import { useRef } from "react";
import { api } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  onClose: () => void;
}

export function PublishPreviewPanel({ moduleId, onClose }: Props) {
  const iframeRef = useRef<HTMLIFrameElement>(null);

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
          <h3 style={{ margin: 0 }}>Document Preview</h3>
          <div style={{ display: "flex", gap: theme.spacing.sm }}>
            <button onClick={handlePrint}>Print</button>
            <button onClick={onClose}>Close</button>
          </div>
        </div>
        <iframe
          ref={iframeRef}
          src={api.getPublishUrl(moduleId, "html")}
          style={{
            flex: 1,
            border: `1px solid ${theme.colors.borderLight}`,
            borderRadius: theme.borderRadius,
          }}
          title="Document Preview"
        />
      </div>
    </div>
  );
}
