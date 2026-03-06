import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { api, type Dashboard, type DashboardWidget as DashboardWidgetType } from "./api/client";
import { DashboardWidget } from "./DashboardWidget";
import { theme } from "./theme";

const WIDGET_TYPES = [
  "coverage_chart",
  "suspect_link_count",
  "lifecycle_distribution",
  "test_status",
];

const WIDGET_TYPE_LABELS: Record<string, string> = {
  coverage_chart: "Coverage Chart",
  suspect_link_count: "Suspect Link Count",
  lifecycle_distribution: "Lifecycle Distribution",
  test_status: "Test Status",
};

const GRID_COLUMNS = 12;
const COLUMN_WIDTH = 80;
const ROW_HEIGHT = 80;
const GAP = 8;

export function DashboardPage() {
  const { workspaceId, dashboardId } = useParams<{ workspaceId: string; dashboardId: string }>();

  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [widgets, setWidgets] = useState<DashboardWidgetType[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [newWidgetType, setNewWidgetType] = useState(WIDGET_TYPES[0]);
  const [newWidgetTitle, setNewWidgetTitle] = useState("");
  const [addingWidget, setAddingWidget] = useState(false);

  const fetchData = async () => {
    if (!workspaceId || !dashboardId) return;
    try {
      setLoading(true);
      const [dashData, widgetData] = await Promise.all([
        api.getDashboard(workspaceId, dashboardId),
        api.listWidgets(dashboardId),
      ]);
      setDashboard(dashData);
      setWidgets(widgetData);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load dashboard");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, [workspaceId, dashboardId]);

  const handleAddWidget = async () => {
    if (!dashboardId || !newWidgetTitle.trim()) return;
    try {
      setAddingWidget(true);
      // Calculate next available position
      const maxY = widgets.reduce((max, w) => Math.max(max, w.position_y + w.height), 0);
      await api.createWidget(dashboardId, {
        widget_type: newWidgetType,
        title: newWidgetTitle.trim(),
        position_x: 0,
        position_y: maxY,
        width: 4,
        height: 3,
      });
      setNewWidgetTitle("");
      await fetchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add widget");
    } finally {
      setAddingWidget(false);
    }
  };

  const handleDeleteWidget = async (widgetId: string) => {
    if (!dashboardId) return;
    if (!confirm("Delete this widget?")) return;
    try {
      await api.deleteWidget(dashboardId, widgetId);
      await fetchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete widget");
    }
  };

  const handleRefresh = () => {
    fetchData();
  };

  if (!workspaceId || !dashboardId) {
    return <div style={{ padding: theme.spacing.lg, color: theme.colors.error }}>Missing route parameters.</div>;
  }

  if (loading) {
    return (
      <div style={{ padding: theme.spacing.lg, textAlign: "center", color: theme.colors.textMuted }}>
        Loading dashboard...
      </div>
    );
  }

  // Compute grid container dimensions
  const gridWidth = GRID_COLUMNS * COLUMN_WIDTH + (GRID_COLUMNS - 1) * GAP;
  const maxRow = widgets.reduce((max, w) => Math.max(max, w.position_y + w.height), 1);
  const gridHeight = maxRow * ROW_HEIGHT + (maxRow - 1) * GAP;

  return (
    <div style={{ padding: theme.spacing.lg, maxWidth: 1100, margin: "0 auto" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: theme.spacing.md }}>
        <div>
          <h2 style={{ margin: 0, color: theme.colors.text }}>
            {dashboard?.name ?? "Dashboard"}
          </h2>
          {dashboard?.description && (
            <div style={{ fontSize: "0.85rem", color: theme.colors.textSecondary, marginTop: 2 }}>
              {dashboard.description}
            </div>
          )}
        </div>
        <a
          href={api.exportDashboardPdfUrl(dashboardId)}
          target="_blank"
          rel="noopener noreferrer"
          style={{
            padding: "6px 16px",
            fontSize: "0.85rem",
            background: theme.colors.primary,
            color: "#fff",
            border: "none",
            borderRadius: theme.borderRadius,
            textDecoration: "none",
            cursor: "pointer",
            whiteSpace: "nowrap",
          }}
        >
          Export PDF
        </a>
      </div>

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

      {/* Add Widget form */}
      <div
        style={{
          padding: theme.spacing.md,
          marginBottom: theme.spacing.lg,
          background: theme.colors.bgCode,
          borderRadius: theme.borderRadius,
          border: `1px solid ${theme.colors.borderLight}`,
        }}
      >
        <h4 style={{ margin: `0 0 ${theme.spacing.sm}`, color: theme.colors.text }}>Add Widget</h4>
        <div style={{ display: "flex", gap: theme.spacing.sm, flexWrap: "wrap", alignItems: "flex-end" }}>
          <div style={{ flex: "1 1 180px" }}>
            <label style={{ display: "block", fontSize: "0.8rem", color: theme.colors.textSecondary, marginBottom: 2 }}>
              Type
            </label>
            <select
              value={newWidgetType}
              onChange={(e) => setNewWidgetType(e.target.value)}
              style={{
                width: "100%",
                padding: "6px 8px",
                fontSize: "0.85rem",
                border: `1px solid ${theme.colors.border}`,
                borderRadius: theme.borderRadius,
                fontFamily: theme.fontFamily,
                background: theme.colors.bg,
                boxSizing: "border-box",
              }}
            >
              {WIDGET_TYPES.map((wt) => (
                <option key={wt} value={wt}>
                  {WIDGET_TYPE_LABELS[wt] ?? wt}
                </option>
              ))}
            </select>
          </div>
          <div style={{ flex: "2 1 250px" }}>
            <label style={{ display: "block", fontSize: "0.8rem", color: theme.colors.textSecondary, marginBottom: 2 }}>
              Title *
            </label>
            <input
              type="text"
              value={newWidgetTitle}
              onChange={(e) => setNewWidgetTitle(e.target.value)}
              placeholder="Widget title"
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
            onClick={handleAddWidget}
            disabled={addingWidget || !newWidgetTitle.trim()}
            style={{
              padding: "6px 16px",
              fontSize: "0.85rem",
              background: theme.colors.primary,
              color: "#fff",
              border: "none",
              borderRadius: theme.borderRadius,
              cursor: addingWidget || !newWidgetTitle.trim() ? "not-allowed" : "pointer",
              opacity: addingWidget || !newWidgetTitle.trim() ? 0.6 : 1,
              whiteSpace: "nowrap",
            }}
          >
            {addingWidget ? "Adding..." : "Add Widget"}
          </button>
        </div>
      </div>

      {/* Widget Grid */}
      {widgets.length === 0 ? (
        <div style={{ textAlign: "center", padding: theme.spacing.xl, color: theme.colors.textMuted }}>
          No widgets yet. Add one above.
        </div>
      ) : (
        <div
          style={{
            position: "relative",
            width: gridWidth,
            height: gridHeight,
            maxWidth: "100%",
          }}
        >
          {widgets.map((widget) => {
            const left = widget.position_x * (COLUMN_WIDTH + GAP);
            const top = widget.position_y * (ROW_HEIGHT + GAP);
            const width = widget.width * COLUMN_WIDTH + (widget.width - 1) * GAP;
            const height = widget.height * ROW_HEIGHT + (widget.height - 1) * GAP;

            return (
              <div
                key={widget.id}
                style={{
                  position: "absolute",
                  left,
                  top,
                  width,
                  height,
                  border: `1px solid ${theme.colors.borderLight}`,
                  borderRadius: theme.borderRadius,
                  background: theme.colors.bg,
                  overflow: "hidden",
                  display: "flex",
                  flexDirection: "column",
                }}
              >
                {/* Widget header bar */}
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: `4px ${theme.spacing.sm}`,
                    background: theme.colors.bgCode,
                    borderBottom: `1px solid ${theme.colors.borderLight}`,
                    flexShrink: 0,
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: theme.spacing.xs, minWidth: 0 }}>
                    <span
                      style={{
                        fontSize: "0.8rem",
                        fontWeight: 600,
                        color: theme.colors.text,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {widget.title}
                    </span>
                    <span
                      style={{
                        fontSize: "0.65rem",
                        padding: "1px 6px",
                        background: theme.colors.primary,
                        color: "#fff",
                        borderRadius: 3,
                        flexShrink: 0,
                      }}
                    >
                      {WIDGET_TYPE_LABELS[widget.widget_type] ?? widget.widget_type}
                    </span>
                  </div>
                  <button
                    onClick={() => handleDeleteWidget(widget.id)}
                    style={{
                      padding: "2px 6px",
                      fontSize: "0.7rem",
                      background: "none",
                      color: theme.colors.danger,
                      border: `1px solid ${theme.colors.danger}`,
                      borderRadius: 3,
                      cursor: "pointer",
                      flexShrink: 0,
                      marginLeft: 4,
                    }}
                  >
                    Delete
                  </button>
                </div>

                {/* Widget content */}
                <div style={{ flex: 1, overflow: "auto", padding: theme.spacing.xs }}>
                  <DashboardWidget
                    dashboardId={dashboardId}
                    widget={widget}
                    onRefresh={handleRefresh}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
