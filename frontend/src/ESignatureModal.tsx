import { useState } from "react";
import { theme } from "./theme";

interface Props {
  onSign: (password: string, meaning: string) => void;
  onCancel: () => void;
  transitionLabel: string;
}

export function ESignatureModal({ onSign, onCancel, transitionLabel }: Props) {
  const [password, setPassword] = useState("");
  const [meaning, setMeaning] = useState(transitionLabel);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!password.trim()) {
      setError("Password is required");
      return;
    }
    if (!meaning.trim()) {
      setError("Meaning is required");
      return;
    }
    onSign(password, meaning.trim());
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
        zIndex: 1200,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onCancel();
      }}
    >
      <div
        style={{
          background: theme.colors.bg,
          borderRadius: 8,
          padding: theme.spacing.lg,
          maxWidth: 440,
          width: "90%",
          boxShadow: "0 8px 32px rgba(0,0,0,0.2)",
        }}
      >
        <h3 style={{ margin: `0 0 ${theme.spacing.md}` }}>
          Electronic Signature Required
        </h3>

        <p
          style={{
            color: theme.colors.textSecondary,
            fontSize: "0.9rem",
            marginBottom: theme.spacing.md,
          }}
        >
          This action requires electronic signature verification. Please
          re-enter your password to confirm.
        </p>

        {error && (
          <div
            style={{
              color: theme.colors.error,
              marginBottom: theme.spacing.sm,
              fontSize: "0.85rem",
            }}
          >
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit}>
          <label
            style={{
              display: "block",
              fontSize: "0.85rem",
              fontWeight: 600,
              marginBottom: 4,
            }}
          >
            Password
          </label>
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoFocus
            style={{
              width: "100%",
              padding: theme.spacing.sm,
              marginBottom: theme.spacing.md,
              boxSizing: "border-box",
            }}
          />

          <label
            style={{
              display: "block",
              fontSize: "0.85rem",
              fontWeight: 600,
              marginBottom: 4,
            }}
          >
            Meaning / Reason
          </label>
          <input
            type="text"
            value={meaning}
            onChange={(e) => setMeaning(e.target.value)}
            style={{
              width: "100%",
              padding: theme.spacing.sm,
              marginBottom: theme.spacing.lg,
              boxSizing: "border-box",
            }}
          />

          <div style={{ display: "flex", gap: theme.spacing.sm, justifyContent: "flex-end" }}>
            <button
              type="button"
              onClick={onCancel}
              style={{
                padding: `${theme.spacing.sm} ${theme.spacing.md}`,
              }}
            >
              Cancel
            </button>
            <button
              type="submit"
              style={{
                padding: `${theme.spacing.sm} ${theme.spacing.md}`,
                background: theme.colors.primary,
                color: "#fff",
                border: "none",
                borderRadius: theme.borderRadius,
                cursor: "pointer",
              }}
            >
              Sign &amp; Confirm
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
