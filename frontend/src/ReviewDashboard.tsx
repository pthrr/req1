import { useEffect, useState } from "react";
import { api, isReviewed, type ReqObject, type ReviewPackage } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  packages: ReviewPackage[];
}

export function ReviewDashboard({ moduleId, packages }: Props) {
  const [objects, setObjects] = useState<ReqObject[]>([]);

  useEffect(() => {
    api.listObjects(moduleId).then((d) => setObjects(d.items)).catch(() => {});
  }, [moduleId]);

  const reviewed = objects.filter((o) => isReviewed(o)).length;
  const unreviewed = objects.length - reviewed;
  const reviewedPct = objects.length > 0 ? Math.round((reviewed / objects.length) * 100) : 0;

  // Package status counts
  const statusCounts: Record<string, number> = {};
  for (const pkg of packages) {
    statusCounts[pkg.status] = (statusCounts[pkg.status] ?? 0) + 1;
  }

  const barWidth = 300;
  const barHeight = 24;
  const reviewedWidth = objects.length > 0 ? (reviewed / objects.length) * barWidth : 0;

  return (
    <div style={{ marginBottom: theme.spacing.lg, padding: theme.spacing.md, background: theme.colors.bgCode, borderRadius: theme.borderRadius }}>
      <h4 style={{ margin: `0 0 ${theme.spacing.sm}` }}>Review Dashboard</h4>

      <div style={{ display: "flex", gap: theme.spacing.lg, alignItems: "flex-start", flexWrap: "wrap" }}>
        {/* Bar chart: reviewed vs unreviewed */}
        <div>
          <div style={{ fontSize: "0.85rem", marginBottom: "4px" }}>
            Object Review Status ({reviewedPct}% reviewed)
          </div>
          <svg width={barWidth} height={barHeight}>
            <rect x={0} y={0} width={barWidth} height={barHeight} fill="#ffcdd2" rx={3} />
            <rect x={0} y={0} width={reviewedWidth} height={barHeight} fill={theme.colors.success} rx={3} />
            {reviewed > 0 && (
              <text x={reviewedWidth / 2} y={barHeight / 2 + 4} textAnchor="middle" fontSize="11" fill="#fff">
                {reviewed}
              </text>
            )}
            {unreviewed > 0 && (
              <text x={reviewedWidth + (barWidth - reviewedWidth) / 2} y={barHeight / 2 + 4} textAnchor="middle" fontSize="11" fill="#333">
                {unreviewed}
              </text>
            )}
          </svg>
          <div style={{ display: "flex", gap: theme.spacing.md, fontSize: "0.8rem", marginTop: "4px" }}>
            <span style={{ color: theme.colors.success }}>Reviewed: {reviewed}</span>
            <span style={{ color: theme.colors.error }}>Unreviewed: {unreviewed}</span>
          </div>
        </div>

        {/* Package status summary */}
        <div>
          <div style={{ fontSize: "0.85rem", marginBottom: "4px" }}>
            Package Status Summary
          </div>
          <div style={{ display: "flex", gap: theme.spacing.sm, flexWrap: "wrap" }}>
            {Object.entries(statusCounts).map(([status, count]) => (
              <div
                key={status}
                style={{
                  padding: "4px 10px",
                  borderRadius: 3,
                  background: theme.colors.bg,
                  border: `1px solid ${theme.colors.borderLight}`,
                  fontSize: "0.85rem",
                }}
              >
                <span style={{ fontWeight: 600 }}>{count}</span>{" "}
                <span style={{ color: theme.colors.textSecondary }}>{status}</span>
              </div>
            ))}
            {packages.length === 0 && (
              <span style={{ fontSize: "0.85rem", color: theme.colors.textMuted }}>No packages</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
