import { useCallback, useEffect, useState } from "react";
import { api, type ChangeProposal, type ReqObject } from "./api/client";
import { InlineDiff } from "./DiffView";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  objects: ReqObject[];
  onObjectsChanged: () => void;
}

interface DiffEntry {
  object_id: string;
  field: string;
  old_value: string;
  new_value: string;
}

export function ChangeProposalPanel({ moduleId, objects, onObjectsChanged }: Props) {
  const [proposals, setProposals] = useState<ChangeProposal[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [newTitle, setNewTitle] = useState("");
  const [newDesc, setNewDesc] = useState("");
  const [applying, setApplying] = useState<string | null>(null);

  const fetchProposals = useCallback(async () => {
    try {
      const data = await api.listChangeProposals(moduleId);
      setProposals(data.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load proposals");
    }
  }, [moduleId]);

  useEffect(() => {
    fetchProposals();
  }, [fetchProposals]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTitle.trim()) return;
    try {
      await api.createChangeProposal(moduleId, {
        title: newTitle.trim(),
        description: newDesc.trim() || undefined,
      });
      setNewTitle("");
      setNewDesc("");
      fetchProposals();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create proposal");
    }
  };

  const handleStatusChange = async (id: string, status: string) => {
    try {
      await api.updateChangeProposal(moduleId, id, { status });
      fetchProposals();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update proposal");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteChangeProposal(moduleId, id);
      if (expandedId === id) setExpandedId(null);
      fetchProposals();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete proposal");
    }
  };

  const handleApply = async (proposal: ChangeProposal) => {
    if (!window.confirm(`Apply approved proposal "${proposal.title}"? This will update objects.`)) return;
    setApplying(proposal.id);
    try {
      const diffData = proposal.diff_data as DiffEntry[] | null;
      if (Array.isArray(diffData)) {
        for (const entry of diffData) {
          const obj = objects.find((o) => o.id === entry.object_id);
          if (obj) {
            await api.updateObject(moduleId, entry.object_id, {
              [entry.field]: entry.new_value,
            } as Record<string, unknown>);
          }
        }
      }
      await api.updateChangeProposal(moduleId, proposal.id, { status: "applied" });
      onObjectsChanged();
      fetchProposals();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to apply proposal");
    } finally {
      setApplying(null);
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case "approved": return theme.colors.success;
      case "rejected": return theme.colors.error;
      case "applied": return theme.colors.primary;
      default: return theme.colors.textMuted;
    }
  };

  return (
    <div>
      {error && <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>{error}</div>}

      <form
        onSubmit={handleCreate}
        style={{ display: "flex", gap: theme.spacing.sm, marginBottom: theme.spacing.md }}
      >
        <input
          type="text"
          value={newTitle}
          onChange={(e) => setNewTitle(e.target.value)}
          placeholder="Proposal title"
          style={{ padding: theme.spacing.sm, flex: 1 }}
        />
        <input
          type="text"
          value={newDesc}
          onChange={(e) => setNewDesc(e.target.value)}
          placeholder="Description (optional)"
          style={{ padding: theme.spacing.sm, flex: 1 }}
        />
        <button type="submit" style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
          Create Proposal
        </button>
      </form>

      {proposals.map((p) => {
        const isExpanded = expandedId === p.id;
        const diffData = p.diff_data as DiffEntry[] | null;

        return (
          <div
            key={p.id}
            style={{
              border: `1px solid ${theme.colors.borderLight}`,
              borderRadius: theme.borderRadius,
              marginBottom: theme.spacing.sm,
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                padding: theme.spacing.sm,
                cursor: "pointer",
                gap: theme.spacing.sm,
              }}
              onClick={() => setExpandedId(isExpanded ? null : p.id)}
            >
              <span style={{ fontSize: "0.7em", width: 16, textAlign: "center", transform: isExpanded ? "rotate(0deg)" : "rotate(-90deg)", transition: "transform 0.15s" }}>
                {"\u25BC"}
              </span>
              <span style={{ fontWeight: 600, flex: 1 }}>{p.title}</span>
              <span style={{ color: statusColor(p.status), fontSize: "0.85rem", fontWeight: 500 }}>
                {p.status}
              </span>
              <span style={{ fontSize: "0.8rem", color: theme.colors.textMuted }}>
                {new Date(p.created_at).toLocaleDateString()}
              </span>
              {p.status === "draft" && (
                <button
                  onClick={(e) => { e.stopPropagation(); handleStatusChange(p.id, "approved"); }}
                  style={{ padding: "2px 8px", fontSize: "0.8rem" }}
                >
                  Approve
                </button>
              )}
              {p.status === "draft" && (
                <button
                  onClick={(e) => { e.stopPropagation(); handleStatusChange(p.id, "rejected"); }}
                  style={{ padding: "2px 8px", fontSize: "0.8rem" }}
                >
                  Reject
                </button>
              )}
              {p.status === "approved" && (
                <button
                  onClick={(e) => { e.stopPropagation(); handleApply(p); }}
                  disabled={applying === p.id}
                  style={{ padding: "2px 8px", fontSize: "0.8rem" }}
                >
                  {applying === p.id ? "Applying..." : "Apply"}
                </button>
              )}
              <button
                onClick={(e) => { e.stopPropagation(); handleDelete(p.id); }}
                style={{ padding: "2px 8px", fontSize: "0.8rem" }}
              >
                Del
              </button>
            </div>

            {isExpanded && (
              <div style={{ padding: `0 ${theme.spacing.sm} ${theme.spacing.sm}`, borderTop: `1px solid ${theme.colors.borderLight}` }}>
                {p.description && (
                  <p style={{ fontSize: "0.85rem", color: theme.colors.textSecondary, margin: `${theme.spacing.sm} 0` }}>
                    {p.description}
                  </p>
                )}
                {Array.isArray(diffData) && diffData.length > 0 ? (
                  <div>
                    <h5 style={{ margin: `${theme.spacing.sm} 0 4px` }}>Changes:</h5>
                    {diffData.map((entry, i) => {
                      const obj = objects.find((o) => o.id === entry.object_id);
                      return (
                        <div key={i} style={{ marginBottom: theme.spacing.sm, padding: theme.spacing.sm, background: theme.colors.bgCode, borderRadius: theme.borderRadius }}>
                          <div style={{ fontSize: "0.8rem", color: theme.colors.textMuted, marginBottom: 4 }}>
                            Object: {obj?.heading ?? entry.object_id.slice(0, 8) + "..."} — Field: {entry.field}
                          </div>
                          <InlineDiff label={entry.field} textA={entry.old_value} textB={entry.new_value} />
                        </div>
                      );
                    })}
                  </div>
                ) : (
                  <p style={{ fontSize: "0.85rem", color: theme.colors.textMuted }}>
                    No diff data attached to this proposal.
                  </p>
                )}
              </div>
            )}
          </div>
        );
      })}

      {proposals.length === 0 && (
        <div style={{ padding: theme.spacing.md, textAlign: "center", color: theme.colors.textMuted }}>
          No change proposals yet.
        </div>
      )}
    </div>
  );
}
