import { useCallback, useEffect, useRef, useState } from "react";
import { type Attachment } from "./api/client";
import { theme } from "./theme";

const BASE_URL = "/api/v1";

interface Props {
  objectId: string;
  onClose: () => void;
}

export function AttachmentPanel({ objectId, onClose }: Props) {
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const fetchAttachments = useCallback(async () => {
    try {
      const res = await fetch(`${BASE_URL}/objects/${objectId}/attachments`);
      if (!res.ok) throw new Error(`Failed: ${res.status}`);
      const data = await res.json();
      setAttachments(data.items ?? data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load attachments");
    }
  }, [objectId]);

  useEffect(() => {
    fetchAttachments();
  }, [fetchAttachments]);

  const handleUpload = async () => {
    const files = fileInputRef.current?.files;
    if (!files || files.length === 0) return;
    setUploading(true);
    try {
      for (const file of Array.from(files)) {
        const formData = new FormData();
        formData.append("file", file);
        const res = await fetch(`${BASE_URL}/objects/${objectId}/attachments`, {
          method: "POST",
          body: formData,
        });
        if (!res.ok) throw new Error(`Upload failed: ${res.status}`);
      }
      if (fileInputRef.current) fileInputRef.current.value = "";
      fetchAttachments();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to upload");
    } finally {
      setUploading(false);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await fetch(`${BASE_URL}/attachments/${id}`, { method: "DELETE" });
      fetchAttachments();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete attachment");
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
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
          maxWidth: 700,
          width: "90%",
          maxHeight: "80vh",
          overflow: "auto",
          boxShadow: "0 8px 32px rgba(0,0,0,0.2)",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: theme.spacing.sm,
          }}
        >
          <h3 style={{ margin: 0 }}>Attachments for {objectId.slice(0, 8)}...</h3>
          <button onClick={onClose}>Close</button>
        </div>

        {error && (
          <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>{error}</div>
        )}

        <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: theme.spacing.md, alignItems: "center" }}>
          <input type="file" ref={fileInputRef} multiple />
          <button
            onClick={handleUpload}
            disabled={uploading}
            style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
          >
            {uploading ? "Uploading..." : "Upload"}
          </button>
        </div>

        {attachments.map((att) => (
          <div
            key={att.id}
            style={{
              display: "flex",
              alignItems: "center",
              gap: theme.spacing.sm,
              padding: theme.spacing.sm,
              borderBottom: `1px solid ${theme.colors.borderLight}`,
            }}
          >
            <span style={{ flex: 1, fontWeight: 500 }}>{att.file_name}</span>
            <span style={{ fontSize: "0.8rem", color: theme.colors.textMuted }}>
              {att.content_type}
            </span>
            <span style={{ fontSize: "0.8rem", color: theme.colors.textMuted }}>
              {formatSize(att.size_bytes)}
            </span>
            <a
              href={`${BASE_URL}/attachments/${att.id}/download`}
              target="_blank"
              rel="noopener noreferrer"
              style={{ fontSize: "0.85rem" }}
            >
              Download
            </a>
            <button
              onClick={() => handleDelete(att.id)}
              style={{ padding: "2px 8px", fontSize: "0.8rem" }}
            >
              Delete
            </button>
          </div>
        ))}

        {attachments.length === 0 && (
          <div style={{ padding: theme.spacing.md, textAlign: "center", color: theme.colors.textMuted }}>
            No attachments yet.
          </div>
        )}
      </div>
    </div>
  );
}
