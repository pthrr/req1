import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router";
import { api, type Dashboard } from "./api/client";
import { theme } from "./theme";

export function DashboardListPage() {
  const { workspaceId } = useParams<{ workspaceId: string }>();
  const navigate = useNavigate();

  const [dashboards, setDashboards] = useState<Dashboard[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [newName, setNewName] = useState("");
  const [newDescription, setNewDescription] = useState("");
  const [creating, setCreating] = useState(false);

  const fetchDashboards = async () => {
    if (!workspaceId) return;
    try {
      setLoading(true);
      const data = await api.listDashboards(workspaceId);
      setDashboards(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load dashboards");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchDashboards();
  }, [workspaceId]);

  const handleCreate = async () => {
    if (!workspaceId || !newName.trim()) return;
    try {
      setCreating(true);
      await api.createDashboard(workspaceId, {
        name: newName.trim(),
        description: newDescription.trim() || undefined,
      });
      setNewName("");
      setNewDescription("");
      await fetchDashboards();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create dashboard");
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (dashboardId: string) => {
    if (!workspaceId) return;
    if (!confirm("Delete this dashboard?")) return;
    try {
      await api.deleteDashboard(workspaceId, dashboardId);
      await fetchDashboards();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete dashboard");
    }
  };

  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return iso;
    }
  };

  if (!workspaceId) {
    return <div style={{ padding: theme.spacing.lg, color: theme.colors.error }}>No workspace ID provided.</div>;
  }

  return (
    <div style={{ padding: theme.spacing.lg, maxWidth: 900, margin: "0 auto" }}>
      <h2 style={{ margin: `0 0 ${theme.spacing.md}`, color: theme.colors.text }}>Dashboards</h2>

      {error && (
        <div
          style={{
            padding: theme.spacing.sm,
            marginBottom: theme.spacing.md,
            background: "#ffebee",
            color: theme.colors.error,
            borderRadius: theme.borderRadius,
            fontSize: "0.85rem",
          }}
        >
          {error}
        </div>
      )}

      {/* Create new dashboard form */}
      <div
        style={{
          padding: theme.spacing.md,
          marginBottom: theme.spacing.lg,
          background: theme.colors.bgCode,
          borderRadius: theme.borderRadius,
          border: `1px solid ${theme.colors.borderLight}`,
        }}
      >
        <h4 style={{ margin: `0 0 ${theme.spacing.sm}`, color: theme.colors.text }}>New Dashboard</h4>
        <div style={{ display: "flex", gap: theme.spacing.sm, flexWrap: "wrap", alignItems: "flex-end" }}>
          <div style={{ flex: "1 1 200px" }}>
            <label style={{ display: "block", fontSize: "0.8rem", color: theme.colors.textSecondary, marginBottom: 2 }}>
              Name *
            </label>
            <input
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Dashboard name"
              style={{
                width: "100%",
                padding: "6px 8px",
                fontSize: "0.85rem",
                border: `1px solid ${theme.colors.border}`,
                borderRadius: theme.borderRadius,
                fontFamily: theme.fontFamily,
                boxSizing: "border-box",
              }}
            />
          </div>
          <div style={{ flex: "2 1 300px" }}>
            <label style={{ display: "block", fontSize: "0.8rem", color: theme.colors.textSecondary, marginBottom: 2 }}>
              Description
            </label>
            <input
              type="text"
              value={newDescription}
              onChange={(e) => setNewDescription(e.target.value)}
              placeholder="Optional description"
              style={{
                width: "100%",
                padding: "6px 8px",
                fontSize: "0.85rem",
                border: `1px solid ${theme.colors.border}`,
                borderRadius: theme.borderRadius,
                fontFamily: theme.fontFamily,
                boxSizing: "border-box",
              }}
            />
          </div>
          <button
            onClick={handleCreate}
            disabled={creating || !newName.trim()}
            style={{
              padding: "6px 16px",
              fontSize: "0.85rem",
              background: theme.colors.primary,
              color: "#fff",
              border: "none",
              borderRadius: theme.borderRadius,
              cursor: creating || !newName.trim() ? "not-allowed" : "pointer",
              opacity: creating || !newName.trim() ? 0.6 : 1,
              whiteSpace: "nowrap",
            }}
          >
            {creating ? "Creating..." : "Create"}
          </button>
        </div>
      </div>

      {/* Dashboard list */}
      {loading ? (
        <div style={{ textAlign: "center", padding: theme.spacing.lg, color: theme.colors.textMuted }}>
          Loading dashboards...
        </div>
      ) : dashboards.length === 0 ? (
        <div style={{ textAlign: "center", padding: theme.spacing.lg, color: theme.colors.textMuted }}>
          No dashboards yet. Create one above.
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: theme.spacing.sm }}>
          {dashboards.map((dashboard) => (
            <div
              key={dashboard.id}
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                padding: theme.spacing.md,
                background: theme.colors.bg,
                border: `1px solid ${theme.colors.borderLight}`,
                borderRadius: theme.borderRadius,
                cursor: "pointer",
                transition: "border-color 0.15s",
              }}
              onClick={() => navigate(`/w/${workspaceId}/dashboards/${dashboard.id}`)}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLDivElement).style.borderColor = theme.colors.primary;
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLDivElement).style.borderColor = theme.colors.borderLight;
              }}
            >
              <div style={{ flex: 1, minWidth: 0 }}>
                <div
                  style={{
                    fontSize: "1rem",
                    fontWeight: 600,
                    color: theme.colors.text,
                    marginBottom: 2,
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {dashboard.name}
                </div>
                {dashboard.description && (
                  <div
                    style={{
                      fontSize: "0.85rem",
                      color: theme.colors.textSecondary,
                      marginBottom: 2,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {dashboard.description}
                  </div>
                )}
                <div style={{ fontSize: "0.75rem", color: theme.colors.textMuted }}>
                  Created {formatDate(dashboard.created_at)}
                </div>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleDelete(dashboard.id);
                }}
                style={{
                  padding: "4px 10px",
                  fontSize: "0.8rem",
                  background: "none",
                  color: theme.colors.danger,
                  border: `1px solid ${theme.colors.danger}`,
                  borderRadius: theme.borderRadius,
                  cursor: "pointer",
                  marginLeft: theme.spacing.sm,
                  flexShrink: 0,
                }}
              >
                Delete
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
