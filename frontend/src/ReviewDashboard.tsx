import { useEffect, useState } from "react";
import { api, isReviewed, type ReqObject, type ReviewPackage, type VotingSummary } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  packages: ReviewPackage[];
}

export function ReviewDashboard({ moduleId, packages }: Props) {
  const [objects, setObjects] = useState<ReqObject[]>([]);
  const [votingSummaries, setVotingSummaries] = useState<VotingSummary[]>([]);

  useEffect(() => {
    api.listObjects(moduleId).then((d) => setObjects(d.items)).catch(() => {});
    api.getVotingSummary(moduleId).then(setVotingSummaries).catch(() => {});
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

        {/* Voting summary per package */}
        {votingSummaries.length > 0 && (
          <div style={{ width: "100%" }}>
            <div style={{ fontSize: "0.85rem", marginBottom: "4px", fontWeight: 600 }}>
              Voting Summary
            </div>
            {votingSummaries.map((vs) => {
              const total = vs.total_assignments;
              if (total === 0) return null;
              const voteBarWidth = 250;
              const approvedW = (vs.approved / total) * voteBarWidth;
              const rejectedW = (vs.rejected / total) * voteBarWidth;
              const abstainedW = (vs.abstained / total) * voteBarWidth;
              const pendingW = (vs.pending / total) * voteBarWidth;
              return (
                <div key={vs.package_id} style={{ marginBottom: theme.spacing.sm }}>
                  <div style={{ fontSize: "0.8rem", marginBottom: 2 }}>
                    {vs.package_name} ({vs.package_status})
                  </div>
                  <svg width={voteBarWidth} height={18}>
                    <rect x={0} y={0} width={approvedW} height={18} fill={theme.colors.success} />
                    <rect x={approvedW} y={0} width={rejectedW} height={18} fill={theme.colors.error} />
                    <rect x={approvedW + rejectedW} y={0} width={abstainedW} height={18} fill="#9e9e9e" />
                    <rect x={approvedW + rejectedW + abstainedW} y={0} width={pendingW} height={18} fill="#ffb300" />
                  </svg>
                  <div style={{ display: "flex", gap: theme.spacing.sm, fontSize: "0.75rem", marginTop: 2 }}>
                    <span style={{ color: theme.colors.success }}>Approved: {vs.approved}</span>
                    <span style={{ color: theme.colors.error }}>Rejected: {vs.rejected}</span>
                    <span style={{ color: "#9e9e9e" }}>Abstained: {vs.abstained}</span>
                    <span style={{ color: "#ffb300" }}>Pending: {vs.pending}</span>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
