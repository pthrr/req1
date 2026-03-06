import { useCallback, useEffect, useState } from "react";
import { api, type AppUser, type ESignature } from "./api/client";
import { theme } from "./theme";

interface Props {
  entityType: string;
  entityId: string;
  users: AppUser[];
}

export function SignatureAuditTrail({ entityType, entityId, users }: Props) {
  const [signatures, setSignatures] = useState<ESignature[]>([]);
  const [expanded, setExpanded] = useState(false);

  const fetchSignatures = useCallback(async () => {
    try {
      const data = await api.listSignatures(entityType, entityId);
      setSignatures(data);
    } catch {
      // Non-critical
    }
  }, [entityType, entityId]);

  useEffect(() => {
    fetchSignatures();
  }, [fetchSignatures]);

  if (signatures.length === 0) return null;

  return (
    <div style={{ marginTop: theme.spacing.sm }}>
      <button
        onClick={() => setExpanded((p) => !p)}
        style={{
          background: "none",
          border: "none",
          cursor: "pointer",
          fontSize: "0.85rem",
          color: theme.colors.primary,
          padding: 0,
          display: "flex",
          alignItems: "center",
          gap: 4,
        }}
      >
        <span
          style={{
            display: "inline-block",
            fontSize: "0.7em",
            transform: expanded ? "rotate(0deg)" : "rotate(-90deg)",
            transition: "transform 0.15s",
          }}
        >
          {"\u25BC"}
        </span>
        Signatures ({signatures.length})
      </button>

      {expanded && (
        <table
          style={{
            width: "100%",
            borderCollapse: "collapse",
            fontSize: "0.8rem",
            marginTop: theme.spacing.xs,
          }}
        >
          <thead>
            <tr>
              <th
                style={{
                  textAlign: "left",
                  borderBottom: `1px solid ${theme.colors.border}`,
                  padding: "4px",
                }}
              >
                Signer
              </th>
              <th
                style={{
                  textAlign: "left",
                  borderBottom: `1px solid ${theme.colors.border}`,
                  padding: "4px",
                }}
              >
                Meaning
              </th>
              <th
                style={{
                  textAlign: "left",
                  borderBottom: `1px solid ${theme.colors.border}`,
                  padding: "4px",
                }}
              >
                Hash
              </th>
              <th
                style={{
                  textAlign: "left",
                  borderBottom: `1px solid ${theme.colors.border}`,
                  padding: "4px",
                }}
              >
                Timestamp
              </th>
            </tr>
          </thead>
          <tbody>
            {signatures.map((sig) => {
              const signer = users.find((u) => u.id === sig.user_id);
              return (
                <tr key={sig.id}>
                  <td
                    style={{
                      padding: "4px",
                      borderBottom: `1px solid ${theme.colors.borderLight}`,
                    }}
                  >
                    {signer?.display_name ?? sig.user_id.slice(0, 8)}
                  </td>
                  <td
                    style={{
                      padding: "4px",
                      borderBottom: `1px solid ${theme.colors.borderLight}`,
                    }}
                  >
                    {sig.meaning}
                  </td>
                  <td
                    style={{
                      padding: "4px",
                      borderBottom: `1px solid ${theme.colors.borderLight}`,
                      fontFamily: "monospace",
                      fontSize: "0.75rem",
                    }}
                    title={sig.signature_hash}
                  >
                    {sig.signature_hash.slice(0, 16)}...
                  </td>
                  <td
                    style={{
                      padding: "4px",
                      borderBottom: `1px solid ${theme.colors.borderLight}`,
                    }}
                  >
                    {new Date(sig.created_at).toLocaleString()}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
}
