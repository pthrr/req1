import { useCallback, useEffect, useState } from "react";
import {
  api,
  type TestCase,
  type TestExecution,
  type TestDashboardSummary,
  type ReqObject,
} from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  objects: ReqObject[];
}

const TEST_TYPES = ["manual", "automated", "exploratory"];
const PRIORITIES = ["critical", "high", "medium", "low"];
const EXEC_STATUSES = ["passed", "failed", "blocked", "skipped", "not_run"];

const STATUS_COLORS: Record<string, string> = {
  passed: "#4caf50",
  failed: "#f44336",
  blocked: "#ff9800",
  skipped: "#9e9e9e",
  not_run: "#2196f3",
};

const PRIORITY_COLORS: Record<string, string> = {
  critical: "#d32f2f",
  high: "#f57c00",
  medium: "#fbc02d",
  low: "#66bb6a",
};

export function TestDashboard({ moduleId, objects }: Props) {
  const [dashboard, setDashboard] = useState<TestDashboardSummary | null>(null);
  const [testCases, setTestCases] = useState<TestCase[]>([]);
  const [expandedCaseId, setExpandedCaseId] = useState<string | null>(null);
  const [executions, setExecutions] = useState<TestExecution[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Create form
  const [newName, setNewName] = useState("");
  const [newDesc, setNewDesc] = useState("");
  const [newType, setNewType] = useState("manual");
  const [newPriority, setNewPriority] = useState("medium");
  const [newReqIds, setNewReqIds] = useState("");

  // Execution form
  const [execStatus, setExecStatus] = useState("passed");
  const [execExecutor, setExecExecutor] = useState("");
  const [execEvidence, setExecEvidence] = useState("");
  const [execEnvironment, setExecEnvironment] = useState("");

  const fetchDashboard = useCallback(async () => {
    try {
      const data = await api.getTestDashboard(moduleId);
      setDashboard(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load dashboard");
    }
  }, [moduleId]);

  const fetchTestCases = useCallback(async () => {
    try {
      const data = await api.listTestCases(moduleId);
      setTestCases(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load test cases");
    }
  }, [moduleId]);

  useEffect(() => {
    fetchDashboard();
    fetchTestCases();
  }, [fetchDashboard, fetchTestCases]);

  const handleCreateTestCase = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim()) return;
    try {
      const reqIds = newReqIds
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      await api.createTestCase(moduleId, {
        name: newName.trim(),
        description: newDesc.trim() || undefined,
        test_type: newType,
        priority: newPriority,
        requirement_ids: reqIds.length > 0 ? reqIds : undefined,
      });
      setNewName("");
      setNewDesc("");
      setNewReqIds("");
      fetchTestCases();
      fetchDashboard();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create test case");
    }
  };

  const handleDeleteTestCase = async (id: string) => {
    try {
      await api.deleteTestCase(moduleId, id);
      if (expandedCaseId === id) setExpandedCaseId(null);
      fetchTestCases();
      fetchDashboard();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete test case");
    }
  };

  const handleExpand = async (tcId: string) => {
    if (expandedCaseId === tcId) {
      setExpandedCaseId(null);
      setExecutions([]);
      return;
    }
    setExpandedCaseId(tcId);
    try {
      const data = await api.listTestExecutions(tcId);
      setExecutions(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load executions");
    }
  };

  const handleRecordExecution = async (testCaseId: string) => {
    try {
      await api.createTestExecution(testCaseId, {
        status: execStatus,
        executor: execExecutor.trim() || undefined,
        evidence: execEvidence.trim() || undefined,
        environment: execEnvironment.trim() || undefined,
        executed_at: new Date().toISOString(),
      });
      setExecExecutor("");
      setExecEvidence("");
      setExecEnvironment("");
      const data = await api.listTestExecutions(testCaseId);
      setExecutions(data);
      fetchDashboard();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to record execution");
    }
  };

  const getObjectHeading = (id: string): string => {
    const obj = objects.find((o) => o.id === id);
    return obj ? (obj.heading ?? obj.level) : id.slice(0, 8) + "...";
  };

  const cov = dashboard?.coverage;
  const byStatus = cov?.by_status;

  return (
    <div>
      {error && (
        <div style={{ color: theme.colors.error, marginBottom: theme.spacing.md }}>{error}</div>
      )}

      {/* Summary Cards */}
      {dashboard && (
        <div style={{ display: "flex", gap: theme.spacing.md, marginBottom: theme.spacing.lg, flexWrap: "wrap" }}>
          <div style={{ background: theme.colors.bgCode, padding: theme.spacing.md, borderRadius: theme.borderRadius, minWidth: 140, textAlign: "center" }}>
            <div style={{ fontSize: "2rem", fontWeight: 700 }}>{dashboard.total_test_cases}</div>
            <div style={{ fontSize: "0.85rem", color: theme.colors.textMuted }}>Test Cases</div>
          </div>
          <div style={{ background: theme.colors.bgCode, padding: theme.spacing.md, borderRadius: theme.borderRadius, minWidth: 140, textAlign: "center" }}>
            <div style={{ fontSize: "2rem", fontWeight: 700, color: "#4caf50" }}>{cov?.test_coverage_pct.toFixed(1)}%</div>
            <div style={{ fontSize: "0.85rem", color: theme.colors.textMuted }}>Test Coverage</div>
          </div>
          <div style={{ background: theme.colors.bgCode, padding: theme.spacing.md, borderRadius: theme.borderRadius, minWidth: 140, textAlign: "center" }}>
            <div style={{ fontSize: "2rem", fontWeight: 700, color: "#2196f3" }}>{cov?.pass_coverage_pct.toFixed(1)}%</div>
            <div style={{ fontSize: "0.85rem", color: theme.colors.textMuted }}>Pass Coverage</div>
          </div>
          <div style={{ background: theme.colors.bgCode, padding: theme.spacing.md, borderRadius: theme.borderRadius, minWidth: 140, textAlign: "center" }}>
            <div style={{ fontSize: "2rem", fontWeight: 700 }}>{cov?.total_requirements}</div>
            <div style={{ fontSize: "0.85rem", color: theme.colors.textMuted }}>Requirements</div>
          </div>
        </div>
      )}

      {/* Status Bar Chart */}
      {byStatus && (
        <div style={{ marginBottom: theme.spacing.lg }}>
          <h3 style={{ marginTop: 0, marginBottom: theme.spacing.sm }}>Execution Status</h3>
          <svg width="100%" height="40" viewBox="0 0 500 40" preserveAspectRatio="none">
            {(() => {
              const total = byStatus.passed + byStatus.failed + byStatus.blocked + byStatus.skipped + byStatus.not_run;
              if (total === 0) return <rect width="500" height="40" fill="#e0e0e0" rx="4" />;
              const segments = [
                { count: byStatus.passed, color: STATUS_COLORS.passed },
                { count: byStatus.failed, color: STATUS_COLORS.failed },
                { count: byStatus.blocked, color: STATUS_COLORS.blocked },
                { count: byStatus.skipped, color: STATUS_COLORS.skipped },
                { count: byStatus.not_run, color: STATUS_COLORS.not_run },
              ];
              let x = 0;
              return segments.map((seg, i) => {
                const w = (seg.count / total) * 500;
                const rect = <rect key={i} x={x} width={w} height="40" fill={seg.color} rx={i === 0 ? 4 : 0} />;
                x += w;
                return rect;
              });
            })()}
          </svg>
          <div style={{ display: "flex", gap: theme.spacing.md, marginTop: theme.spacing.sm, fontSize: "0.85rem", flexWrap: "wrap" }}>
            {EXEC_STATUSES.map((s) => (
              <span key={s} style={{ display: "flex", alignItems: "center", gap: 4 }}>
                <span style={{ display: "inline-block", width: 12, height: 12, borderRadius: 2, background: STATUS_COLORS[s] }} />
                {s.replace("_", " ")}: {byStatus[s as keyof typeof byStatus]}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Coverage Progress Bars */}
      {cov && (
        <div style={{ marginBottom: theme.spacing.lg }}>
          <h3 style={{ marginTop: 0, marginBottom: theme.spacing.sm }}>Coverage</h3>
          <div style={{ marginBottom: theme.spacing.sm }}>
            <div style={{ display: "flex", justifyContent: "space-between", fontSize: "0.85rem", marginBottom: 2 }}>
              <span>Test Coverage ({cov.requirements_with_tests}/{cov.total_requirements})</span>
              <span>{cov.test_coverage_pct.toFixed(1)}%</span>
            </div>
            <div style={{ height: 20, background: "#e0e0e0", borderRadius: 4, overflow: "hidden" }}>
              <div style={{ height: "100%", width: `${cov.test_coverage_pct}%`, background: "#4caf50", borderRadius: 4 }} />
            </div>
          </div>
          <div>
            <div style={{ display: "flex", justifyContent: "space-between", fontSize: "0.85rem", marginBottom: 2 }}>
              <span>Pass Coverage ({cov.requirements_with_passing_tests}/{cov.total_requirements})</span>
              <span>{cov.pass_coverage_pct.toFixed(1)}%</span>
            </div>
            <div style={{ height: 20, background: "#e0e0e0", borderRadius: 4, overflow: "hidden" }}>
              <div style={{ height: "100%", width: `${cov.pass_coverage_pct}%`, background: "#2196f3", borderRadius: 4 }} />
            </div>
          </div>
        </div>
      )}

      {/* Create Test Case Form */}
      <h3>Test Cases</h3>
      <form onSubmit={handleCreateTestCase} style={{ display: "flex", gap: theme.spacing.sm, marginBottom: theme.spacing.md, flexWrap: "wrap" }}>
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="Test case name"
          style={{ padding: theme.spacing.sm, flex: 1, minWidth: 200 }}
        />
        <input
          type="text"
          value={newDesc}
          onChange={(e) => setNewDesc(e.target.value)}
          placeholder="Description"
          style={{ padding: theme.spacing.sm, flex: 1, minWidth: 200 }}
        />
        <select value={newType} onChange={(e) => setNewType(e.target.value)} style={{ padding: theme.spacing.sm }}>
          {TEST_TYPES.map((t) => <option key={t} value={t}>{t}</option>)}
        </select>
        <select value={newPriority} onChange={(e) => setNewPriority(e.target.value)} style={{ padding: theme.spacing.sm }}>
          {PRIORITIES.map((p) => <option key={p} value={p}>{p}</option>)}
        </select>
        <input
          type="text"
          value={newReqIds}
          onChange={(e) => setNewReqIds(e.target.value)}
          placeholder="Requirement IDs (comma-sep)"
          style={{ padding: theme.spacing.sm, minWidth: 200 }}
        />
        <button type="submit" style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
          Add Test Case
        </button>
      </form>

      {/* Test Cases Table */}
      <table style={{ width: "100%", borderCollapse: "collapse" }}>
        <thead>
          <tr>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: theme.spacing.sm }}>Name</th>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: theme.spacing.sm }}>Type</th>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: theme.spacing.sm }}>Priority</th>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: theme.spacing.sm }}>Status</th>
            <th style={{ textAlign: "left", borderBottom: "2px solid #ccc", padding: theme.spacing.sm }}>Requirements</th>
            <th style={{ borderBottom: "2px solid #ccc", padding: theme.spacing.sm }} />
          </tr>
        </thead>
        <tbody>
          {testCases.map((tc) => (
            <>
              <tr key={tc.id}>
                <td style={{ padding: theme.spacing.sm, borderBottom: "1px solid #eee" }}>
                  <button
                    onClick={() => handleExpand(tc.id)}
                    style={{ background: "none", border: "none", cursor: "pointer", fontWeight: 600, color: theme.colors.primary, padding: 0, textAlign: "left" }}
                  >
                    {expandedCaseId === tc.id ? "\u25BC" : "\u25B6"} {tc.name}
                  </button>
                </td>
                <td style={{ padding: theme.spacing.sm, borderBottom: "1px solid #eee" }}>
                  <code>{tc.test_type}</code>
                </td>
                <td style={{ padding: theme.spacing.sm, borderBottom: "1px solid #eee" }}>
                  <span style={{
                    display: "inline-block",
                    padding: "2px 8px",
                    borderRadius: 10,
                    fontSize: "0.8rem",
                    fontWeight: 600,
                    color: "#fff",
                    background: PRIORITY_COLORS[tc.priority] ?? "#999",
                  }}>
                    {tc.priority}
                  </span>
                </td>
                <td style={{ padding: theme.spacing.sm, borderBottom: "1px solid #eee" }}>
                  <span style={{
                    display: "inline-block",
                    padding: "2px 8px",
                    borderRadius: 10,
                    fontSize: "0.8rem",
                    fontWeight: 600,
                    color: tc.status === "ready" ? "#2e7d32" : tc.status === "deprecated" ? "#c62828" : "#555",
                    background: tc.status === "ready" ? "#e8f5e9" : tc.status === "deprecated" ? "#ffebee" : "#f5f5f5",
                  }}>
                    {tc.status}
                  </span>
                </td>
                <td style={{ padding: theme.spacing.sm, borderBottom: "1px solid #eee", fontSize: "0.85rem" }}>
                  {tc.requirement_ids.length > 0
                    ? tc.requirement_ids.map((rid) => getObjectHeading(rid)).join(", ")
                    : <span style={{ color: theme.colors.textMuted }}>none</span>}
                </td>
                <td style={{ padding: theme.spacing.sm, borderBottom: "1px solid #eee", whiteSpace: "nowrap" }}>
                  <button onClick={() => handleDeleteTestCase(tc.id)} style={{ padding: "2px 8px", fontSize: "0.8rem" }}>
                    Delete
                  </button>
                </td>
              </tr>
              {expandedCaseId === tc.id && (
                <tr key={`${tc.id}-exec`}>
                  <td colSpan={6} style={{ padding: theme.spacing.md, background: theme.colors.bgCode, borderBottom: "1px solid #eee" }}>
                    {tc.description && <p style={{ margin: `0 0 ${theme.spacing.sm}`, fontSize: "0.9rem" }}>{tc.description}</p>}

                    {/* Record Execution Form */}
                    <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: theme.spacing.md, flexWrap: "wrap", alignItems: "center" }}>
                      <strong style={{ fontSize: "0.85rem" }}>Record Execution:</strong>
                      <select value={execStatus} onChange={(e) => setExecStatus(e.target.value)} style={{ padding: "4px" }}>
                        {EXEC_STATUSES.map((s) => <option key={s} value={s}>{s.replace("_", " ")}</option>)}
                      </select>
                      <input
                        type="text"
                        value={execExecutor}
                        onChange={(e) => setExecExecutor(e.target.value)}
                        placeholder="Executor"
                        style={{ padding: "4px", width: 120 }}
                      />
                      <input
                        type="text"
                        value={execEnvironment}
                        onChange={(e) => setExecEnvironment(e.target.value)}
                        placeholder="Environment"
                        style={{ padding: "4px", width: 120 }}
                      />
                      <input
                        type="text"
                        value={execEvidence}
                        onChange={(e) => setExecEvidence(e.target.value)}
                        placeholder="Evidence / notes"
                        style={{ padding: "4px", flex: 1, minWidth: 150 }}
                      />
                      <button
                        onClick={() => handleRecordExecution(tc.id)}
                        style={{ padding: "4px 12px", fontSize: "0.85rem" }}
                      >
                        Record
                      </button>
                    </div>

                    {/* Execution History */}
                    <h4 style={{ margin: `0 0 ${theme.spacing.sm}` }}>Execution History</h4>
                    {executions.length === 0 ? (
                      <div style={{ color: theme.colors.textMuted, fontSize: "0.85rem" }}>No executions recorded yet.</div>
                    ) : (
                      <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.85rem" }}>
                        <thead>
                          <tr>
                            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "4px 8px" }}>Status</th>
                            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "4px 8px" }}>Executor</th>
                            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "4px 8px" }}>Environment</th>
                            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "4px 8px" }}>Date</th>
                            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "4px 8px" }}>Evidence</th>
                          </tr>
                        </thead>
                        <tbody>
                          {executions.map((ex) => (
                            <tr key={ex.id}>
                              <td style={{ padding: "4px 8px", borderBottom: "1px solid #eee" }}>
                                <span style={{
                                  display: "inline-block",
                                  padding: "1px 6px",
                                  borderRadius: 8,
                                  fontSize: "0.75rem",
                                  fontWeight: 600,
                                  color: "#fff",
                                  background: STATUS_COLORS[ex.status] ?? "#999",
                                }}>
                                  {ex.status.replace("_", " ")}
                                </span>
                              </td>
                              <td style={{ padding: "4px 8px", borderBottom: "1px solid #eee" }}>{ex.executor ?? "—"}</td>
                              <td style={{ padding: "4px 8px", borderBottom: "1px solid #eee" }}>{ex.environment ?? "—"}</td>
                              <td style={{ padding: "4px 8px", borderBottom: "1px solid #eee" }}>
                                {ex.executed_at ? new Date(ex.executed_at).toLocaleString() : ex.created_at ? new Date(ex.created_at).toLocaleString() : "—"}
                              </td>
                              <td style={{ padding: "4px 8px", borderBottom: "1px solid #eee" }}>{ex.evidence ?? "—"}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    )}
                  </td>
                </tr>
              )}
            </>
          ))}
          {testCases.length === 0 && (
            <tr>
              <td colSpan={6} style={{ padding: theme.spacing.lg, textAlign: "center", color: theme.colors.textMuted }}>
                No test cases yet. Create one above.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
