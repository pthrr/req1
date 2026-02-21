import { useCallback, useEffect, useState } from "react";
import { api, type Comment } from "./api/client";
import { theme } from "./theme";

interface Props {
  objectId: string;
  onClose: () => void;
}

export function CommentPanel({ objectId, onClose }: Props) {
  const [comments, setComments] = useState<Comment[]>([]);
  const [newBody, setNewBody] = useState("");
  const [error, setError] = useState<string | null>(null);

  const fetchComments = useCallback(async () => {
    try {
      const data = await api.listComments(objectId);
      const sorted = data.items.sort(
        (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
      );
      setComments(sorted);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load comments");
    }
  }, [objectId]);

  useEffect(() => {
    fetchComments();
  }, [fetchComments]);

  const handleAdd = async () => {
    if (!newBody.trim()) return;
    try {
      await api.createComment(objectId, { body: newBody.trim() });
      setNewBody("");
      await fetchComments();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add comment");
    }
  };

  const handleToggleResolved = async (c: Comment) => {
    try {
      await api.updateComment(objectId, c.id, { resolved: !c.resolved });
      await fetchComments();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update comment");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteComment(objectId, id);
      await fetchComments();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete comment");
    }
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
          <h3 style={{ margin: 0 }}>Comments for {objectId.slice(0, 8)}...</h3>
          <button onClick={onClose}>Close</button>
        </div>

        {error && <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>{error}</div>}

        <div style={{ marginBottom: theme.spacing.md }}>
          {comments.map((c) => (
            <div
              key={c.id}
              style={{
                padding: theme.spacing.sm,
                borderBottom: `1px solid ${theme.colors.borderLight}`,
                opacity: c.resolved ? 0.6 : 1,
              }}
            >
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: theme.spacing.sm }}>
                <div style={{ flex: 1 }}>
                  <p style={{ margin: "0 0 0.25rem 0", whiteSpace: "pre-wrap" }}>{c.body}</p>
                  <span style={{ fontSize: "0.8rem", color: theme.colors.textMuted }}>
                    {new Date(c.created_at).toLocaleString()}
                  </span>
                </div>
                <div style={{ display: "flex", gap: theme.spacing.xs, flexShrink: 0 }}>
                  <button
                    onClick={() => handleToggleResolved(c)}
                    style={{
                      padding: "0.2rem 0.5rem",
                      fontSize: "0.8rem",
                      background: c.resolved ? theme.colors.success : "transparent",
                      color: c.resolved ? "#fff" : theme.colors.text,
                      border: `1px solid ${c.resolved ? theme.colors.success : theme.colors.border}`,
                      borderRadius: 4,
                      cursor: "pointer",
                    }}
                  >
                    {c.resolved ? "Resolved" : "Resolve"}
                  </button>
                  <button
                    onClick={() => handleDelete(c.id)}
                    style={{ padding: "0.2rem 0.5rem", fontSize: "0.8rem" }}
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>
          ))}
          {comments.length === 0 && (
            <div style={{ padding: theme.spacing.md, textAlign: "center", color: theme.colors.textMuted }}>
              No comments yet.
            </div>
          )}
        </div>

        <div style={{ display: "flex", gap: theme.spacing.sm }}>
          <textarea
            value={newBody}
            onChange={(e) => setNewBody(e.target.value)}
            placeholder="Write a comment..."
            rows={3}
            style={{ flex: 1, padding: theme.spacing.sm, resize: "vertical" }}
          />
          <button
            onClick={handleAdd}
            style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, alignSelf: "flex-end" }}
          >
            Add Comment
          </button>
        </div>
      </div>
    </div>
  );
}
