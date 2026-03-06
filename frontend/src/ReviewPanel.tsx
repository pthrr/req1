import { useCallback, useEffect, useState } from "react";
import {
  api,
  type AppUser,
  type Module,
  type ReviewAssignment,
  type ReviewComment,
  type ReviewPackage,
} from "./api/client";
import { ReviewDashboard } from "./ReviewDashboard";
import { MentionTextarea, renderMentionText } from "./MentionTextarea";
import { ESignatureModal } from "./ESignatureModal";
import { SignatureAuditTrail } from "./SignatureAuditTrail";
import { theme } from "./theme";

interface Props {
  moduleId: string;
}

const STATUS_FLOW = ["draft", "open", "in_review", "approved", "rejected"] as const;
const TRANSITIONS: Record<string, string[]> = {
  draft: ["open"],
  open: ["in_review"],
  in_review: ["approved", "rejected"],
  approved: [],
  rejected: ["draft"],
};

const statusColor = (status: string) => {
  switch (status) {
    case "approved": return theme.colors.success;
    case "rejected": return theme.colors.error;
    case "in_review": return theme.colors.primary;
    case "open": return "#f57c00";
    default: return theme.colors.textMuted;
  }
};

export function ReviewPanel({ moduleId }: Props) {
  const [packages, setPackages] = useState<ReviewPackage[]>([]);
  const [users, setUsers] = useState<AppUser[]>([]);
  const [expandedPkg, setExpandedPkg] = useState<string | null>(null);
  const [assignments, setAssignments] = useState<Record<string, ReviewAssignment[]>>({});
  const [comments, setComments] = useState<Record<string, ReviewComment[]>>({});
  const [newCommentBody, setNewCommentBody] = useState("");
  const [newPkgName, setNewPkgName] = useState("");
  const [newPkgDesc, setNewPkgDesc] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [mod, setMod] = useState<Module | null>(null);
  const [sigModal, setSigModal] = useState<{ pkg: ReviewPackage; status: string } | null>(null);

  const fetchPackages = useCallback(async () => {
    try {
      const data = await api.listReviewPackages(moduleId);
      setPackages(data.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load review packages");
    }
  }, [moduleId]);

  const fetchUsers = useCallback(async () => {
    try {
      const data = await api.listUsers({ active: true });
      setUsers(data.items);
    } catch {
      // Non-critical
    }
  }, []);

  const fetchModule = useCallback(async () => {
    try {
      const data = await api.getModule(moduleId);
      setMod(data);
    } catch {
      // Non-critical
    }
  }, [moduleId]);

  useEffect(() => {
    fetchPackages();
    fetchUsers();
    fetchModule();
  }, [fetchPackages, fetchUsers, fetchModule]);

  const fetchAssignments = useCallback(async (packageId: string) => {
    try {
      const data = await api.listReviewAssignments(packageId);
      setAssignments((prev) => ({ ...prev, [packageId]: data.items }));
    } catch {
      // Non-critical
    }
  }, []);

  const fetchComments = useCallback(async (packageId: string) => {
    try {
      const data = await api.listReviewComments(packageId);
      setComments((prev) => ({ ...prev, [packageId]: data.items }));
    } catch {
      // Non-critical
    }
  }, []);

  const handleExpand = (pkgId: string) => {
    if (expandedPkg === pkgId) {
      setExpandedPkg(null);
    } else {
      setExpandedPkg(pkgId);
      if (!assignments[pkgId]) fetchAssignments(pkgId);
      if (!comments[pkgId]) fetchComments(pkgId);
    }
  };

  const handleCreatePackage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newPkgName.trim()) return;
    try {
      await api.createReviewPackage(moduleId, {
        name: newPkgName.trim(),
        description: newPkgDesc.trim() || undefined,
      });
      setNewPkgName("");
      setNewPkgDesc("");
      fetchPackages();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create review package");
    }
  };

  const handleDeletePackage = async (id: string) => {
    try {
      await api.deleteReviewPackage(moduleId, id);
      if (expandedPkg === id) setExpandedPkg(null);
      fetchPackages();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete review package");
    }
  };

  const handleStatusTransition = async (pkg: ReviewPackage, newStatus: string) => {
    // Check if signature is required
    const transitions = mod?.signature_config?.require_signature_transitions ?? [];
    const transitionKey = `${pkg.status}->${newStatus}`;
    if (transitions.includes(transitionKey)) {
      setSigModal({ pkg, status: newStatus });
      return;
    }
    try {
      await api.transitionReviewPackage(moduleId, pkg.id, { status: newStatus });
      fetchPackages();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update status");
    }
  };

  const handleSignedTransition = async (password: string, meaning: string) => {
    if (!sigModal) return;
    try {
      await api.transitionReviewPackage(moduleId, sigModal.pkg.id, {
        status: sigModal.status,
        password,
        meaning,
      });
      setSigModal(null);
      fetchPackages();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to transition with signature");
      setSigModal(null);
    }
  };

  const handleAddAssignment = async (packageId: string, reviewerId?: string) => {
    try {
      await api.createReviewAssignment(packageId, { reviewer_id: reviewerId });
      fetchAssignments(packageId);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add assignment");
    }
  };

  const handleUpdateAssignment = async (packageId: string, assignmentId: string, data: { status?: string; comment?: string }) => {
    try {
      await api.updateReviewAssignment(packageId, assignmentId, data);
      fetchAssignments(packageId);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update assignment");
    }
  };

  const handleDeleteAssignment = async (packageId: string, assignmentId: string) => {
    try {
      await api.deleteReviewAssignment(packageId, assignmentId);
      fetchAssignments(packageId);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete assignment");
    }
  };

  return (
    <div>
      {error && <div style={{ color: theme.colors.error, marginBottom: theme.spacing.sm }}>{error}</div>}

      <ReviewDashboard moduleId={moduleId} packages={packages} />

      <form
        onSubmit={handleCreatePackage}
        style={{ display: "flex", gap: theme.spacing.sm, marginBottom: theme.spacing.md }}
      >
        <input
          type="text"
          value={newPkgName}
          onChange={(e) => setNewPkgName(e.target.value)}
          placeholder="Package name"
          style={{ padding: theme.spacing.sm, flex: 1 }}
        />
        <input
          type="text"
          value={newPkgDesc}
          onChange={(e) => setNewPkgDesc(e.target.value)}
          placeholder="Description (optional)"
          style={{ padding: theme.spacing.sm, flex: 1 }}
        />
        <button type="submit" style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
          Create Package
        </button>
      </form>

      {packages.map((pkg) => {
        const isExpanded = expandedPkg === pkg.id;
        const pkgAssignments = assignments[pkg.id] ?? [];
        const transitions = TRANSITIONS[pkg.status] ?? [];

        return (
          <div
            key={pkg.id}
            style={{
              border: `1px solid ${theme.colors.borderLight}`,
              borderRadius: theme.borderRadius,
              marginBottom: theme.spacing.sm,
            }}
          >
            {/* Package header */}
            <div
              style={{
                display: "flex",
                alignItems: "center",
                padding: theme.spacing.sm,
                cursor: "pointer",
                gap: theme.spacing.sm,
              }}
              onClick={() => handleExpand(pkg.id)}
            >
              <span style={{ fontSize: "0.7em", width: 16, textAlign: "center", transform: isExpanded ? "rotate(0deg)" : "rotate(-90deg)", transition: "transform 0.15s" }}>
                {"\u25BC"}
              </span>
              <span style={{ fontWeight: 600, flex: 1 }}>{pkg.name}</span>

              {/* Status step indicator */}
              <div style={{ display: "flex", gap: 2, alignItems: "center" }}>
                {STATUS_FLOW.map((s) => (
                  <span
                    key={s}
                    style={{
                      padding: "1px 6px",
                      fontSize: "0.7rem",
                      borderRadius: 3,
                      background: s === pkg.status ? statusColor(s) : theme.colors.bgCode,
                      color: s === pkg.status ? "#fff" : theme.colors.textMuted,
                    }}
                  >
                    {s}
                  </span>
                ))}
              </div>

              {/* Transition buttons */}
              {transitions.map((t) => (
                <button
                  key={t}
                  onClick={(e) => { e.stopPropagation(); handleStatusTransition(pkg, t); }}
                  style={{ padding: "2px 8px", fontSize: "0.8rem" }}
                >
                  {t === "approved" ? "Approve" : t === "rejected" ? "Reject" : t.charAt(0).toUpperCase() + t.slice(1)}
                </button>
              ))}

              <button
                onClick={(e) => { e.stopPropagation(); handleDeletePackage(pkg.id); }}
                style={{ padding: "2px 8px", fontSize: "0.8rem" }}
              >
                Delete
              </button>
            </div>

            {/* Expanded: assignments */}
            {isExpanded && (
              <div style={{ padding: `0 ${theme.spacing.sm} ${theme.spacing.sm}`, borderTop: `1px solid ${theme.colors.borderLight}` }}>
                {pkg.description && (
                  <p style={{ fontSize: "0.85rem", color: theme.colors.textSecondary, margin: `${theme.spacing.sm} 0` }}>
                    {pkg.description}
                  </p>
                )}

                <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.85rem", marginBottom: theme.spacing.sm }}>
                  <thead>
                    <tr>
                      <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "4px" }}>Reviewer</th>
                      <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "4px" }}>Status</th>
                      <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "4px" }}>Comment</th>
                      <th style={{ textAlign: "left", borderBottom: `1px solid ${theme.colors.border}`, padding: "4px" }}>Signed</th>
                      <th style={{ borderBottom: `1px solid ${theme.colors.border}`, padding: "4px" }} />
                    </tr>
                  </thead>
                  <tbody>
                    {pkgAssignments.map((a) => {
                      const reviewer = users.find((u) => u.id === a.reviewer_id);
                      return (
                        <tr key={a.id}>
                          <td style={{ padding: "4px", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                            {reviewer?.display_name ?? a.reviewer_id?.slice(0, 8) ?? "(unassigned)"}
                          </td>
                          <td style={{ padding: "4px", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                            <span style={{ color: statusColor(a.status) }}>{a.status}</span>
                          </td>
                          <td style={{ padding: "4px", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                            <input
                              type="text"
                              defaultValue={a.comment ?? ""}
                              onBlur={(e) => {
                                if (e.target.value !== (a.comment ?? "")) {
                                  handleUpdateAssignment(pkg.id, a.id, { comment: e.target.value });
                                }
                              }}
                              style={{ width: "100%", padding: "2px 4px", fontSize: "0.85rem", boxSizing: "border-box" }}
                            />
                          </td>
                          <td style={{ padding: "4px", borderBottom: `1px solid ${theme.colors.borderLight}`, fontSize: "0.8rem" }}>
                            {a.signed_at ? new Date(a.signed_at).toLocaleString() : "-"}
                          </td>
                          <td style={{ padding: "4px", borderBottom: `1px solid ${theme.colors.borderLight}` }}>
                            <div style={{ display: "flex", gap: 4 }}>
                              {a.status === "pending" && (
                                <>
                                  <button
                                    onClick={() => handleUpdateAssignment(pkg.id, a.id, { status: "approved" })}
                                    style={{ padding: "2px 6px", fontSize: "0.75rem", color: theme.colors.success }}
                                  >
                                    Approve
                                  </button>
                                  <button
                                    onClick={() => handleUpdateAssignment(pkg.id, a.id, { status: "rejected" })}
                                    style={{ padding: "2px 6px", fontSize: "0.75rem", color: theme.colors.error }}
                                  >
                                    Reject
                                  </button>
                                  <button
                                    onClick={() => handleUpdateAssignment(pkg.id, a.id, { status: "abstained" })}
                                    style={{ padding: "2px 6px", fontSize: "0.75rem" }}
                                  >
                                    Abstain
                                  </button>
                                </>
                              )}
                              <button
                                onClick={() => handleDeleteAssignment(pkg.id, a.id)}
                                style={{ padding: "2px 6px", fontSize: "0.75rem" }}
                              >
                                Del
                              </button>
                            </div>
                          </td>
                        </tr>
                      );
                    })}
                    {pkgAssignments.length === 0 && (
                      <tr>
                        <td colSpan={5} style={{ padding: "8px", textAlign: "center", color: theme.colors.textMuted }}>
                          No assignments yet.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>

                <div style={{ display: "flex", gap: theme.spacing.sm, alignItems: "center" }}>
                  <span style={{ fontSize: "0.85rem" }}>Add reviewer:</span>
                  <select
                    onChange={(e) => {
                      if (e.target.value) {
                        handleAddAssignment(pkg.id, e.target.value);
                        e.target.value = "";
                      }
                    }}
                    style={{ padding: "4px" }}
                  >
                    <option value="">Select user...</option>
                    {users.map((u) => (
                      <option key={u.id} value={u.id}>{u.display_name}</option>
                    ))}
                  </select>
                </div>

                {/* Discussion chat */}
                <div style={{ marginTop: theme.spacing.md, borderTop: `1px solid ${theme.colors.borderLight}`, paddingTop: theme.spacing.sm }}>
                  <h5 style={{ margin: `0 0 ${theme.spacing.sm}` }}>Discussion</h5>
                  <div style={{ maxHeight: 200, overflowY: "auto", marginBottom: theme.spacing.sm }}>
                    {(comments[pkg.id] ?? []).map((c) => {
                      const author = users.find((u) => u.id === c.author_id);
                      return (
                        <div key={c.id} style={{ marginBottom: theme.spacing.sm, fontSize: "0.85rem" }}>
                          <div style={{ display: "flex", gap: theme.spacing.sm, alignItems: "baseline" }}>
                            <span style={{ fontWeight: 600 }}>{author?.display_name ?? "Anonymous"}</span>
                            <span style={{ fontSize: "0.75rem", color: theme.colors.textMuted }}>
                              {new Date(c.created_at).toLocaleString()}
                            </span>
                          </div>
                          <div>{renderMentionText(c.body, users)}</div>
                        </div>
                      );
                    })}
                    {(comments[pkg.id] ?? []).length === 0 && (
                      <div style={{ color: theme.colors.textMuted, fontSize: "0.85rem" }}>No messages yet.</div>
                    )}
                  </div>
                  <div style={{ display: "flex", gap: theme.spacing.sm }}>
                    <MentionTextarea
                      value={expandedPkg === pkg.id ? newCommentBody : ""}
                      onChange={(v) => setNewCommentBody(v)}
                      users={users}
                      placeholder="Type a message... (use @ to mention)"
                      rows={2}
                      style={{ fontSize: "0.85rem" }}
                    />
                    <button
                      onClick={() => {
                        if (!newCommentBody.trim()) return;
                        api.createReviewComment(pkg.id, { body: newCommentBody.trim() })
                          .then(() => { setNewCommentBody(""); fetchComments(pkg.id); })
                          .catch((err) => setError(err instanceof Error ? err.message : "Failed to send"));
                      }}
                      style={{ padding: "4px 12px", fontSize: "0.85rem", alignSelf: "flex-end" }}
                    >
                      Send
                    </button>
                  </div>
                </div>

                <SignatureAuditTrail
                  entityType="review_package"
                  entityId={pkg.id}
                  users={users}
                />
              </div>
            )}
          </div>
        );
      })}

      {sigModal && (
        <ESignatureModal
          transitionLabel={`${sigModal.pkg.status} -> ${sigModal.status}`}
          onSign={handleSignedTransition}
          onCancel={() => setSigModal(null)}
        />
      )}

      {packages.length === 0 && (
        <div style={{ padding: theme.spacing.md, textAlign: "center", color: theme.colors.textMuted }}>
          No review packages yet. Create one to start the review process.
        </div>
      )}
    </div>
  );
}
