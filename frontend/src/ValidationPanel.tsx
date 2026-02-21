import { useCallback, useState } from "react";
import { api, type ValidationIssue, type ValidationReport } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
}

const SEVERITY_COLORS: Record<string, string> = {
  error: theme.colors.error,
  warning: theme.colors.suspect,
  info: theme.colors.primary,
};

export function ValidationPanel({ moduleId }: Props) {
  const [report, setReport] = useState<ValidationReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterRule, setFilterRule] = useState<string>("");
  const [filterSeverity, setFilterSeverity] = useState<string>("");

  const runValidation = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.validateModule(moduleId);
      setReport(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Validation failed");
    } finally {
      setLoading(false);
    }
  }, [moduleId]);

  const filteredIssues = report?.issues.filter((issue) => {
    if (filterRule && issue.rule !== filterRule) return false;
    if (filterSeverity && issue.severity !== filterSeverity) return false;
    return true;
  }) ?? [];

  const ruleNames = report
    ? [...new Set(report.issues.map((i) => i.rule))].sort()
    : [];

  const severityCounts = report
    ? report.issues.reduce<Record<string, number>>((acc, i) => {
        acc[i.severity] = (acc[i.severity] ?? 0) + 1;
        return acc;
      }, {})
    : {};

  return (
    <div>
      <div style={{ display: "flex", gap: theme.spacing.sm, alignItems: "center", marginBottom: theme.spacing.md }}>
        <button
          onClick={runValidation}
          disabled={loading}
          style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
        >
          {loading ? "Validating..." : "Run Validation"}
        </button>
        {report && (
          <span style={{ color: theme.colors.textSecondary, fontSize: "0.9rem" }}>
            {report.object_count} objects, {report.link_count} links checked
          </span>
        )}
      </div>

      {error && (
        <div style={{ color: theme.colors.error, marginBottom: theme.spacing.md }}>{error}</div>
      )}

      {report && (
        <>
          {/* Summary badges */}
          <div style={{ display: "flex", gap: theme.spacing.md, marginBottom: theme.spacing.md }}>
            {(["error", "warning", "info"] as const).map((sev) => (
              <span
                key={sev}
                style={{
                  padding: `${theme.spacing.xs} ${theme.spacing.sm}`,
                  borderRadius: theme.borderRadius,
                  background: SEVERITY_COLORS[sev] ?? theme.colors.textMuted,
                  color: "#fff",
                  fontSize: "0.85rem",
                  cursor: "pointer",
                  opacity: filterSeverity && filterSeverity !== sev ? 0.4 : 1,
                }}
                onClick={() => setFilterSeverity(filterSeverity === sev ? "" : sev)}
              >
                {severityCounts[sev] ?? 0} {sev}{(severityCounts[sev] ?? 0) !== 1 ? "s" : ""}
              </span>
            ))}
          </div>

          {/* Filters */}
          {ruleNames.length > 1 && (
            <div style={{ marginBottom: theme.spacing.sm }}>
              <select
                value={filterRule}
                onChange={(e) => setFilterRule(e.target.value)}
                style={{ padding: theme.spacing.xs }}
              >
                <option value="">All rules</option>
                {ruleNames.map((r) => (
                  <option key={r} value={r}>{r}</option>
                ))}
              </select>
            </div>
          )}

          {/* Issues list */}
          {report.issues.length === 0 ? (
            <div style={{ color: theme.colors.success, padding: theme.spacing.md, fontSize: "1.1rem" }}>
              All checks passed.
            </div>
          ) : (
            <div style={{ maxHeight: 500, overflowY: "auto" }}>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.9rem" }}>
                <thead>
                  <tr style={{ borderBottom: `2px solid ${theme.colors.borderLight}`, textAlign: "left" }}>
                    <th style={{ padding: theme.spacing.xs, width: 80 }}>Severity</th>
                    <th style={{ padding: theme.spacing.xs, width: 150 }}>Rule</th>
                    <th style={{ padding: theme.spacing.xs }}>Message</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredIssues.map((issue, i) => (
                    <IssueRow key={i} issue={issue} />
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </>
      )}
    </div>
  );
}

function IssueRow({ issue }: { issue: ValidationIssue }) {
  return (
    <tr style={{ borderBottom: `1px solid ${theme.colors.borderLight}` }}>
      <td style={{ padding: theme.spacing.xs }}>
        <span
          style={{
            color: SEVERITY_COLORS[issue.severity] ?? theme.colors.text,
            fontWeight: 600,
            fontSize: "0.85rem",
          }}
        >
          {issue.severity}
        </span>
      </td>
      <td style={{ padding: theme.spacing.xs, color: theme.colors.textSecondary }}>
        {issue.rule}
      </td>
      <td style={{ padding: theme.spacing.xs }}>
        {issue.message}
      </td>
    </tr>
  );
}
