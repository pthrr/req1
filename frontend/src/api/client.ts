const BASE_URL = "/api/v1";

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  offset: number;
  limit: number;
}

export interface Module {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  prefix: string;
  separator: string;
  digits: number;
  required_attributes: string[];
  default_classification: string;
  publish_template: string | null;
  default_lifecycle_model_id: string | null;
  signature_config: SignatureConfig;
  created_at: string;
  updated_at: string;
}

export interface SignatureConfig {
  require_signature_transitions?: string[];
  require_four_eyes?: boolean;
}

export interface ReqObject {
  id: string;
  module_id: string;
  parent_id: string | null;
  position: number;
  level: string;
  heading: string | null;
  body: string | null;
  attributes: Record<string, unknown> | null;
  current_version: number;
  classification: string;
  content_fingerprint: string;
  reviewed_fingerprint: string | null;
  reviewed_at: string | null;
  reviewed_by: string | null;
  references_: unknown[];
  object_type_id: string | null;
  lifecycle_state: string | null;
  lifecycle_model_id: string | null;
  source_object_id: string | null;
  source_module_id: string | null;
  is_placeholder: boolean;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
}

export function isReviewed(obj: ReqObject): boolean {
  return obj.reviewed_fingerprint != null && obj.reviewed_fingerprint === obj.content_fingerprint;
}

export interface ObjectHistory {
  id: number;
  object_id: string;
  module_id: string;
  version: number;
  attribute_values: Record<string, unknown> | null;
  heading: string | null;
  body: string | null;
  changed_by: string | null;
  changed_at: string;
  change_type: string;
}

export interface LinkType {
  id: string;
  name: string;
  description: string | null;
  created_at: string;
}

export interface Link {
  id: string;
  source_object_id: string;
  target_object_id: string;
  link_type_id: string;
  attributes: Record<string, unknown> | null;
  suspect: boolean;
  source_fingerprint: string;
  target_fingerprint: string;
  created_at: string;
  updated_at: string;
}

export interface Baseline {
  id: string;
  module_id: string;
  name: string;
  description: string | null;
  created_by: string | null;
  created_at: string;
  locked: boolean;
  baseline_set_id: string | null;
}

export interface BaselineEntry {
  baseline_id: string;
  object_id: string;
  version: number;
}

export interface BaselineWithEntries extends Baseline {
  entries: BaselineEntry[];
}

export interface BaselineDiffEntry {
  object_id: string;
  version: number;
  heading: string | null;
  body: string | null;
  attributes: Record<string, unknown> | null;
}

export interface BaselineDiffModified {
  object_id: string;
  version_a: number;
  version_b: number;
  heading_a: string | null;
  heading_b: string | null;
  body_a: string | null;
  body_b: string | null;
  attributes_a: Record<string, unknown> | null;
  attributes_b: Record<string, unknown> | null;
}

export interface BaselineDiff {
  baseline_a: string;
  baseline_b: string;
  added: BaselineDiffEntry[];
  removed: BaselineDiffEntry[];
  modified: BaselineDiffModified[];
}

export interface Workspace {
  id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface Project {
  id: string;
  workspace_id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface AttributeDefinition {
  id: string;
  module_id: string | null;
  name: string;
  data_type: string;
  default_value: string | null;
  enum_values: string[] | null;
  multi_select: boolean;
  depends_on: string | null;
  dependency_mapping: Record<string, string[]> | null;
  created_at: string;
}

export interface MatrixObject {
  id: string;
  heading: string | null;
  position: number;
}

export interface MatrixCell {
  source_id: string;
  target_id: string;
  link_id: string;
  suspect: boolean;
}

export interface TraceabilityMatrixResponse {
  source_objects: MatrixObject[];
  target_objects: MatrixObject[];
  cells: MatrixCell[];
}

export interface Script {
  id: string;
  module_id: string;
  name: string;
  script_type: string;
  hook_point: string | null;
  source_code: string;
  enabled: boolean;
  priority: number;
  cron_expression: string | null;
  last_run_at: string | null;
  next_run_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface ValidationIssue {
  rule: string;
  severity: string;
  object_id: string | null;
  link_id: string | null;
  message: string;
}

export interface ValidationReport {
  module_id: string;
  issues: ValidationIssue[];
  object_count: number;
  link_count: number;
}

export interface ScriptTestResult {
  script_type: string;
  // trigger
  rejected?: boolean;
  reason?: string | null;
  // layout
  value?: string;
  // action/trigger
  output?: string[];
  mutations?: unknown[];
}

export interface ScriptExecuteResult {
  output: string[];
  mutations_applied: number;
}

export interface View {
  id: string;
  module_id: string;
  name: string;
  column_config: unknown;
  filter_config: unknown;
  sort_config: unknown;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface ObjectType {
  id: string;
  module_id: string;
  name: string;
  description: string | null;
  default_classification: string;
  required_attributes: unknown;
  attribute_schema: unknown;
  created_at: string;
  updated_at: string;
}

export interface Comment {
  id: string;
  object_id: string;
  author_id: string | null;
  body: string;
  mentioned_user_ids: string[];
  resolved: boolean;
  created_at: string;
  updated_at: string;
}

export interface AppUser {
  id: string;
  email: string;
  display_name: string;
  role: string;
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface ReviewPackage {
  id: string;
  module_id: string;
  name: string;
  description: string | null;
  status: string;
  created_by: string | null;
  created_at: string;
  updated_at: string;
}

export interface ReviewAssignment {
  id: string;
  package_id: string;
  reviewer_id: string | null;
  status: string;
  comment: string | null;
  signed_at: string | null;
  created_at: string;
}

export interface ReviewComment {
  id: string;
  package_id: string;
  author_id: string | null;
  body: string;
  mentioned_user_ids: string[];
  created_at: string;
  updated_at: string;
}

export interface VotingSummary {
  package_id: string;
  package_name: string;
  package_status: string;
  total_assignments: number;
  approved: number;
  rejected: number;
  abstained: number;
  pending: number;
}

export interface ChangeProposal {
  id: string;
  module_id: string;
  title: string;
  description: string | null;
  status: string;
  author_id: string | null;
  diff_data: unknown;
  created_at: string;
  updated_at: string;
}

export interface BaselineSet {
  id: string;
  name: string;
  version: string;
  description: string | null;
  created_by: string | null;
  created_at: string;
}

export interface Attachment {
  id: string;
  object_id: string;
  file_name: string;
  content_type: string;
  size_bytes: number;
  storage_path: string;
  sha256: string | null;
  created_at: string;
}

export interface Webhook {
  id: string;
  module_id: string;
  name: string;
  url: string;
  secret: string | null;
  events: string;
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface LifecycleModel {
  id: string;
  module_id: string;
  name: string;
  description: string | null;
  initial_state: string;
  states: Array<{ name: string; color?: string; description?: string }>;
  transitions: Array<{ from: string; to: string }>;
  created_at: string;
  updated_at: string;
}

export interface ImpactObject {
  id: string;
  heading: string | null;
  level: string;
  depth: number;
  link_type: string | null;
  module_id: string;
}

export interface ImpactEdge {
  source_id: string;
  target_id: string;
  link_type: string | null;
  suspect: boolean;
}

export interface ImpactResponse {
  root_id: string;
  direction: string;
  max_depth: number;
  objects: ImpactObject[];
  edges: ImpactEdge[];
}

export interface CoverageResponse {
  total_objects: number;
  with_upstream: number;
  with_downstream: number;
  with_any_link: number;
  upstream_pct: number;
  downstream_pct: number;
  any_link_pct: number;
}

export interface LayoutEntry {
  object_id: string;
  value: string;
}

export interface BatchLayoutResponse {
  results: LayoutEntry[];
}

export interface TestCase {
  id: string;
  module_id: string;
  name: string;
  description: string | null;
  preconditions: string | null;
  expected_result: string | null;
  test_type: string;
  priority: string;
  status: string;
  requirement_ids: string[];
  created_at: string;
  updated_at: string;
}

export interface TestExecution {
  id: string;
  test_case_id: string;
  status: string;
  executor: string | null;
  executed_at: string | null;
  duration_ms: number | null;
  evidence: string | null;
  environment: string | null;
  created_at: string;
}

export interface TestStatusCounts {
  passed: number;
  failed: number;
  blocked: number;
  skipped: number;
  not_run: number;
}

export interface TestCoverageResponse {
  total_requirements: number;
  requirements_with_tests: number;
  requirements_with_passing_tests: number;
  test_coverage_pct: number;
  pass_coverage_pct: number;
  total_test_cases: number;
  by_status: TestStatusCounts;
}

export interface TestCaseStatusCounts {
  draft: number;
  ready: number;
  deprecated: number;
}

export interface TestPriorityCounts {
  critical: number;
  high: number;
  medium: number;
  low: number;
}

export interface TestDashboardSummary {
  total_test_cases: number;
  by_test_status: TestCaseStatusCounts;
  by_priority: TestPriorityCounts;
  recent_executions: TestExecution[];
  coverage: TestCoverageResponse;
}

export interface Diagram {
  id: string;
  module_id: string;
  name: string;
  description: string | null;
  diagram_type: string;
  source_code: string;
  linked_object_ids: string[];
  created_by: string | null;
  created_at: string;
  updated_at: string;
}

export interface ScriptExecution {
  id: string;
  script_id: string;
  status: string;
  started_at: string;
  finished_at: string | null;
  duration_ms: number | null;
  output: string | null;
  error_message: string | null;
  created_at: string;
}

export interface AuditLogEntry {
  id: number;
  user_id: string | null;
  action: string;
  entity_type: string;
  entity_id: string | null;
  details: unknown | null;
  ip_address: string | null;
  created_at: string;
}

export interface Notification {
  id: string;
  user_id: string;
  notification_type: string;
  title: string;
  body: string;
  entity_type: string;
  entity_id: string | null;
  read: boolean;
  created_at: string;
}

export interface ESignature {
  id: string;
  user_id: string;
  entity_type: string;
  entity_id: string;
  meaning: string;
  signature_hash: string;
  ip_address: string | null;
  created_at: string;
}

export interface Dashboard {
  id: string;
  workspace_id: string;
  name: string;
  description: string | null;
  layout: unknown;
  created_by: string | null;
  created_at: string;
  updated_at: string;
}

export interface DashboardWidget {
  id: string;
  dashboard_id: string;
  widget_type: string;
  title: string;
  config: Record<string, unknown>;
  position_x: number;
  position_y: number;
  width: number;
  height: number;
  created_at: string;
  updated_at: string;
}

export interface WidgetDataEntry {
  label: string;
  value: number;
  extra: Record<string, unknown> | null;
}

export interface ProjectTemplate {
  id: string;
  name: string;
  description: string | null;
  standard: string | null;
  version: string | null;
  template_data: unknown;
  is_builtin: boolean;
  created_by: string | null;
  created_at: string;
  updated_at: string;
}

export interface InstantiateResult {
  project_id: string;
  modules_created: number;
}

export interface DocxPreviewResult {
  styles: Array<{ style_id: string; sample_text: string; count: number }>;
  paragraph_count: number;
}

export interface DocxImportResult {
  objects_created: number;
  objects_updated: number;
  paragraphs_skipped: number;
}

export interface FormLayout {
  sections: FormSection[];
}

export interface FormSection {
  id: string;
  title: string;
  columns: number;
  fields: FormField[];
}

export interface FormField {
  attribute_name: string;
  order: number;
  width?: string;
  required?: boolean;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const token = localStorage.getItem("token");
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(init?.headers as Record<string, string> ?? {}),
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(`${BASE_URL}${path}`, {
    ...init,
    headers,
  });

  if (res.status === 401) {
    localStorage.removeItem("token");
    if (window.location.pathname !== "/login") {
      window.location.href = "/login";
    }
    throw new Error("Session expired, please log in again");
  }

  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new Error(body?.error?.message ?? `Request failed: ${res.status}`);
  }

  if (res.status === 204) return undefined as T;
  return res.json();
}

export const api = {
  // --- Modules ---
  listModules: (filters?: { project_id?: string; limit?: number }) => {
    const params = new URLSearchParams();
    if (filters?.project_id) params.set("project_id", filters.project_id);
    if (filters?.limit) params.set("limit", String(filters.limit));
    const qs = params.toString();
    return request<PaginatedResponse<Module>>(`/modules${qs ? `?${qs}` : ""}`);
  },

  createModule: (data: {
    name: string;
    description?: string;
    project_id?: string;
    prefix?: string;
    separator?: string;
    digits?: number;
    required_attributes?: string[];
    default_classification?: string;
  }) =>
    request<Module>("/modules", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getModule: (id: string) => request<Module>(`/modules/${id}`),

  updateModule: (id: string, data: {
    name?: string;
    description?: string;
    prefix?: string;
    separator?: string;
    digits?: number;
    required_attributes?: string[];
    default_classification?: string;
    publish_template?: string;
    default_lifecycle_model_id?: string;
    signature_config?: SignatureConfig;
  }) =>
    request<Module>(`/modules/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteModule: (id: string) =>
    request<void>(`/modules/${id}`, { method: "DELETE" }),

  // --- Objects ---
  listObjects: (
    moduleId: string,
    filters?: {
      heading?: string;
      body?: string;
      search?: string;
      sort_by?: string;
      sort_dir?: string;
      needs_review?: string;
    },
  ) => {
    const params = new URLSearchParams();
    if (filters?.heading) params.set("heading", filters.heading);
    if (filters?.body) params.set("body", filters.body);
    if (filters?.search) params.set("search", filters.search);
    if (filters?.sort_by) params.set("sort_by", filters.sort_by);
    if (filters?.sort_dir) params.set("sort_dir", filters.sort_dir);
    if (filters?.needs_review) params.set("needs_review", filters.needs_review);
    const qs = params.toString();
    return request<PaginatedResponse<ReqObject>>(
      `/modules/${moduleId}/objects${qs ? `?${qs}` : ""}`,
    );
  },

  createObject: (
    moduleId: string,
    data: { heading?: string; body?: string; position?: number; parent_id?: string; object_type_id?: string; classification?: string; source_object_id?: string; source_module_id?: string; is_placeholder?: boolean },
  ) =>
    request<ReqObject>(`/modules/${moduleId}/objects`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getObject: (moduleId: string, id: string) =>
    request<ReqObject>(`/modules/${moduleId}/objects/${id}`),

  updateObject: (
    moduleId: string,
    id: string,
    data: {
      heading?: string;
      body?: string;
      position?: number;
      parent_id?: string | null;
      attributes?: Record<string, unknown>;
      reviewed?: boolean;
      classification?: string;
      references?: unknown;
      object_type_id?: string | null;
      expected_version?: number;
      lifecycle_state?: string;
    },
  ) =>
    request<ReqObject>(`/modules/${moduleId}/objects/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteObject: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/objects/${id}`, { method: "DELETE" }),

  moveObject: (
    moduleId: string,
    objectId: string,
    action:
      | { action: "up" }
      | { action: "down" }
      | { action: "indent" }
      | { action: "dedent" }
      | { action: "move_to"; parent_id: string | null; position: number },
  ) =>
    request<ReqObject>(`/modules/${moduleId}/objects/${objectId}/move`, {
      method: "POST",
      body: JSON.stringify(action),
    }),

  // --- Object History ---
  listObjectHistory: (moduleId: string, objectId: string) =>
    request<PaginatedResponse<ObjectHistory>>(
      `/modules/${moduleId}/objects/${objectId}/history`,
    ),

  // --- Link Types ---
  listLinkTypes: () => request<LinkType[]>("/link-types"),

  createLinkType: (data: { name: string; description?: string }) =>
    request<LinkType>("/link-types", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  // --- Links ---
  listLinks: (filters?: {
    source_object_id?: string;
    target_object_id?: string;
    link_type_id?: string;
    module_id?: string;
  }) => {
    const params = new URLSearchParams();
    if (filters?.source_object_id)
      params.set("source_object_id", filters.source_object_id);
    if (filters?.target_object_id)
      params.set("target_object_id", filters.target_object_id);
    if (filters?.link_type_id)
      params.set("link_type_id", filters.link_type_id);
    if (filters?.module_id) params.set("module_id", filters.module_id);
    const qs = params.toString();
    return request<PaginatedResponse<Link>>(`/links${qs ? `?${qs}` : ""}`);
  },

  createLink: (data: {
    source_object_id: string;
    target_object_id: string;
    link_type_id: string;
  }) =>
    request<Link>("/links", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getLink: (id: string) => request<Link>(`/links/${id}`),

  updateLink: (id: string, data: { suspect?: boolean }) =>
    request<Link>(`/links/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteLink: (id: string) =>
    request<void>(`/links/${id}`, { method: "DELETE" }),

  // --- Baselines ---
  listBaselines: (moduleId: string) =>
    request<PaginatedResponse<Baseline>>(`/modules/${moduleId}/baselines`),

  createBaseline: (moduleId: string, data: { name: string; description?: string; baseline_set_id?: string }) =>
    request<BaselineWithEntries>(`/modules/${moduleId}/baselines`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getBaseline: (moduleId: string, id: string) =>
    request<BaselineWithEntries>(`/modules/${moduleId}/baselines/${id}`),

  deleteBaseline: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/baselines/${id}`, { method: "DELETE" }),

  diffBaselines: (moduleId: string, a: string, b: string) =>
    request<BaselineDiff>(
      `/modules/${moduleId}/baseline-diff?a=${encodeURIComponent(a)}&b=${encodeURIComponent(b)}`,
    ),

  diffBaselinesGlobal: (a: string, b: string) =>
    request<BaselineDiff>(
      `/baseline-diff?a=${encodeURIComponent(a)}&b=${encodeURIComponent(b)}`,
    ),

  listAllBaselines: (limit?: number) => {
    const params = new URLSearchParams();
    if (limit != null) params.set("limit", String(limit));
    return request<PaginatedResponse<Baseline>>(`/baselines?${params.toString()}`);
  },

  // --- Workspaces ---
  listWorkspaces: () => request<PaginatedResponse<Workspace>>("/workspaces?limit=500"),

  createWorkspace: (data: { name: string; description?: string }) =>
    request<Workspace>("/workspaces", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getWorkspace: (id: string) => request<Workspace>(`/workspaces/${id}`),

  updateWorkspace: (id: string, data: { name?: string; description?: string }) =>
    request<Workspace>(`/workspaces/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteWorkspace: (id: string) =>
    request<void>(`/workspaces/${id}`, { method: "DELETE" }),

  // --- Projects ---
  listProjects: (workspaceId: string) =>
    request<PaginatedResponse<Project>>(
      `/workspaces/${workspaceId}/projects?limit=500`,
    ),

  createProject: (workspaceId: string, data: { name: string; description?: string }) =>
    request<Project>(`/workspaces/${workspaceId}/projects`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getProject: (workspaceId: string, id: string) =>
    request<Project>(`/workspaces/${workspaceId}/projects/${id}`),

  updateProject: (
    workspaceId: string,
    id: string,
    data: { name?: string; description?: string },
  ) =>
    request<Project>(`/workspaces/${workspaceId}/projects/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteProject: (workspaceId: string, id: string) =>
    request<void>(`/workspaces/${workspaceId}/projects/${id}`, {
      method: "DELETE",
    }),

  // --- Attribute Definitions ---
  listAttributeDefinitions: (moduleId: string) =>
    request<PaginatedResponse<AttributeDefinition>>(
      `/modules/${moduleId}/attribute-definitions`,
    ),

  createAttributeDefinition: (
    moduleId: string,
    data: { name: string; data_type: string; default_value?: string; enum_values?: string[]; depends_on?: string; dependency_mapping?: Record<string, string[]> },
  ) =>
    request<AttributeDefinition>(`/modules/${moduleId}/attribute-definitions`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  updateAttributeDefinition: (
    moduleId: string,
    id: string,
    data: { name?: string; data_type?: string; default_value?: string; enum_values?: string[]; depends_on?: string | null; dependency_mapping?: Record<string, string[]> },
  ) =>
    request<AttributeDefinition>(`/modules/${moduleId}/attribute-definitions/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteAttributeDefinition: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/attribute-definitions/${id}`, {
      method: "DELETE",
    }),

  // --- Scripts ---
  listScripts: (moduleId: string) =>
    request<Script[]>(`/modules/${moduleId}/scripts`),

  createScript: (
    moduleId: string,
    data: {
      name: string;
      script_type?: string;
      hook_point?: string;
      source_code: string;
      enabled?: boolean;
      priority?: number;
      cron_expression?: string;
    },
  ) =>
    request<Script>(`/modules/${moduleId}/scripts`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  updateScript: (
    moduleId: string,
    id: string,
    data: {
      name?: string;
      script_type?: string;
      hook_point?: string;
      source_code?: string;
      enabled?: boolean;
      priority?: number;
      cron_expression?: string;
    },
  ) =>
    request<Script>(`/modules/${moduleId}/scripts/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteScript: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/scripts/${id}`, { method: "DELETE" }),

  testScript: (
    moduleId: string,
    id: string,
    data?: { object?: unknown; hook_point?: string },
  ) =>
    request<ScriptTestResult>(`/modules/${moduleId}/scripts/${id}/test`, {
      method: "POST",
      body: JSON.stringify(data ?? {}),
    }),

  executeScript: (moduleId: string, id: string) =>
    request<ScriptExecuteResult>(`/modules/${moduleId}/scripts/${id}/execute`, {
      method: "POST",
    }),

  // --- Validation ---
  validateModule: (moduleId: string) =>
    request<ValidationReport>(`/modules/${moduleId}/validate`),

  // --- Publishing ---
  getPublishUrl: (moduleId: string, format: string = "html") =>
    `${BASE_URL}/modules/${moduleId}/publish?format=${format}`,

  importCsv: async (moduleId: string, csvContent: string): Promise<{ objects_created: number }> => {
    const token = localStorage.getItem("token");
    const headers: Record<string, string> = { "Content-Type": "text/csv" };
    if (token) headers["Authorization"] = `Bearer ${token}`;
    const res = await fetch(`${BASE_URL}/modules/${moduleId}/import/csv`, {
      method: "POST",
      headers,
      body: csvContent,
    });
    if (!res.ok) {
      const body = await res.json().catch(() => null);
      throw new Error(body?.error?.message ?? `Import failed: ${res.status}`);
    }
    return res.json();
  },

  // --- Traceability Matrix ---
  getTraceabilityMatrix: (
    sourceModuleId: string,
    targetModuleId: string,
    linkTypeId?: string,
  ) => {
    const params = new URLSearchParams();
    params.set("source_module_id", sourceModuleId);
    params.set("target_module_id", targetModuleId);
    if (linkTypeId) params.set("link_type_id", linkTypeId);
    return request<TraceabilityMatrixResponse>(
      `/traceability-matrix?${params.toString()}`,
    );
  },

  // --- Views ---
  listViews: (moduleId: string) =>
    request<PaginatedResponse<View>>(`/modules/${moduleId}/views`),

  createView: (moduleId: string, data: { name: string; column_config?: unknown; filter_config?: unknown; sort_config?: unknown; is_default?: boolean }) =>
    request<View>(`/modules/${moduleId}/views`, { method: "POST", body: JSON.stringify(data) }),

  getView: (moduleId: string, id: string) =>
    request<View>(`/modules/${moduleId}/views/${id}`),

  updateView: (moduleId: string, id: string, data: { name?: string; column_config?: unknown; filter_config?: unknown; sort_config?: unknown; is_default?: boolean }) =>
    request<View>(`/modules/${moduleId}/views/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteView: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/views/${id}`, { method: "DELETE" }),

  // --- Object Types ---
  listObjectTypes: (moduleId: string) => {
    const params = new URLSearchParams();
    params.set("module_id", moduleId);
    return request<PaginatedResponse<ObjectType>>(`/object-types?${params.toString()}`);
  },

  createObjectType: (data: { module_id: string; name: string; description?: string; default_classification?: string; required_attributes?: unknown; attribute_schema?: unknown }) =>
    request<ObjectType>("/object-types", { method: "POST", body: JSON.stringify(data) }),

  getObjectType: (id: string) =>
    request<ObjectType>(`/object-types/${id}`),

  updateObjectType: (id: string, data: { name?: string; description?: string; default_classification?: string; required_attributes?: unknown; attribute_schema?: unknown }) =>
    request<ObjectType>(`/object-types/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteObjectType: (id: string) =>
    request<void>(`/object-types/${id}`, { method: "DELETE" }),

  // --- Comments ---
  listComments: (objectId: string) =>
    request<PaginatedResponse<Comment>>(`/objects/${objectId}/comments`),

  createComment: (objectId: string, data: { body: string; author_id?: string }) =>
    request<Comment>(`/objects/${objectId}/comments`, { method: "POST", body: JSON.stringify(data) }),

  getComment: (objectId: string, id: string) =>
    request<Comment>(`/objects/${objectId}/comments/${id}`),

  updateComment: (objectId: string, id: string, data: { body?: string; resolved?: boolean }) =>
    request<Comment>(`/objects/${objectId}/comments/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteComment: (objectId: string, id: string) =>
    request<void>(`/objects/${objectId}/comments/${id}`, { method: "DELETE" }),

  // --- Users ---
  listUsers: (filters?: { active?: boolean }) => {
    const params = new URLSearchParams();
    if (filters?.active !== undefined) params.set("active", String(filters.active));
    const qs = params.toString();
    return request<PaginatedResponse<AppUser>>(`/users${qs ? `?${qs}` : ""}`);
  },

  createUser: (data: { email: string; display_name: string; role?: string }) =>
    request<AppUser>("/users", { method: "POST", body: JSON.stringify(data) }),

  getUser: (id: string) => request<AppUser>(`/users/${id}`),

  updateUser: (id: string, data: { display_name?: string; role?: string; active?: boolean }) =>
    request<AppUser>(`/users/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteUser: (id: string) =>
    request<void>(`/users/${id}`, { method: "DELETE" }),

  // --- Review Packages ---
  listReviewPackages: (moduleId: string) =>
    request<PaginatedResponse<ReviewPackage>>(`/modules/${moduleId}/review-packages`),

  createReviewPackage: (moduleId: string, data: { name: string; description?: string }) =>
    request<ReviewPackage>(`/modules/${moduleId}/review-packages`, { method: "POST", body: JSON.stringify(data) }),

  getReviewPackage: (moduleId: string, id: string) =>
    request<ReviewPackage>(`/modules/${moduleId}/review-packages/${id}`),

  updateReviewPackage: (moduleId: string, id: string, data: { name?: string; description?: string; status?: string }) =>
    request<ReviewPackage>(`/modules/${moduleId}/review-packages/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteReviewPackage: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/review-packages/${id}`, { method: "DELETE" }),

  // --- Review Assignments ---
  listReviewAssignments: (packageId: string) =>
    request<PaginatedResponse<ReviewAssignment>>(`/review-packages/${packageId}/assignments`),

  createReviewAssignment: (packageId: string, data: { reviewer_id?: string }) =>
    request<ReviewAssignment>(`/review-packages/${packageId}/assignments`, { method: "POST", body: JSON.stringify(data) }),

  getReviewAssignment: (packageId: string, id: string) =>
    request<ReviewAssignment>(`/review-packages/${packageId}/assignments/${id}`),

  updateReviewAssignment: (packageId: string, id: string, data: { status?: string; comment?: string }) =>
    request<ReviewAssignment>(`/review-packages/${packageId}/assignments/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteReviewAssignment: (packageId: string, id: string) =>
    request<void>(`/review-packages/${packageId}/assignments/${id}`, { method: "DELETE" }),

  // --- Change Proposals ---
  listChangeProposals: (moduleId: string) =>
    request<PaginatedResponse<ChangeProposal>>(`/modules/${moduleId}/change-proposals`),

  createChangeProposal: (moduleId: string, data: { title: string; description?: string; diff_data?: unknown }) =>
    request<ChangeProposal>(`/modules/${moduleId}/change-proposals`, { method: "POST", body: JSON.stringify(data) }),

  getChangeProposal: (moduleId: string, id: string) =>
    request<ChangeProposal>(`/modules/${moduleId}/change-proposals/${id}`),

  updateChangeProposal: (moduleId: string, id: string, data: { title?: string; description?: string; status?: string; diff_data?: unknown }) =>
    request<ChangeProposal>(`/modules/${moduleId}/change-proposals/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteChangeProposal: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/change-proposals/${id}`, { method: "DELETE" }),

  // --- Baseline Sets ---
  listBaselineSets: () =>
    request<PaginatedResponse<BaselineSet>>("/baseline-sets"),

  createBaselineSet: (data: { name: string; version: string; description?: string }) =>
    request<BaselineSet>("/baseline-sets", { method: "POST", body: JSON.stringify(data) }),

  getBaselineSet: (id: string) =>
    request<BaselineSet>(`/baseline-sets/${id}`),

  updateBaselineSet: (id: string, data: { name?: string; version?: string; description?: string }) =>
    request<BaselineSet>(`/baseline-sets/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteBaselineSet: (id: string) =>
    request<void>(`/baseline-sets/${id}`, { method: "DELETE" }),

  // --- Impact Analysis ---
  getImpact: (objectId: string, direction?: string, maxDepth?: number) => {
    const params = new URLSearchParams();
    if (direction) params.set("direction", direction);
    if (maxDepth != null) params.set("max_depth", String(maxDepth));
    const qs = params.toString();
    return request<ImpactResponse>(`/object-impact/${objectId}${qs ? `?${qs}` : ""}`);
  },

  // --- Batch Layout ---
  batchLayout: (moduleId: string, scriptId: string) =>
    request<BatchLayoutResponse>(`/modules/${moduleId}/scripts/${scriptId}/layout`, {
      method: "POST",
    }),

  // --- Module Templates ---
  createModuleFromTemplate: (data: {
    name: string;
    project_id: string;
    description?: string;
    template_module_id: string;
    copy_objects?: boolean;
  }) =>
    request<Module>("/modules/from-template", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  // --- Coverage ---
  getCoverage: (moduleId: string) =>
    request<CoverageResponse>(`/modules/${moduleId}/coverage`),

  // --- Attachments ---
  listAttachments: (objectId: string) =>
    request<Attachment[]>(`/objects/${objectId}/attachments`),

  uploadAttachment: async (objectId: string, file: File): Promise<Attachment> => {
    const formData = new FormData();
    formData.append("file", file);
    const token = localStorage.getItem("token");
    const headers: Record<string, string> = {};
    if (token) headers["Authorization"] = `Bearer ${token}`;
    const res = await fetch(`${BASE_URL}/objects/${objectId}/attachments`, {
      method: "POST",
      headers,
      body: formData,
    });
    if (!res.ok) {
      const body = await res.json().catch(() => null);
      throw new Error(body?.error?.message ?? `Upload failed: ${res.status}`);
    }
    return res.json();
  },

  downloadAttachmentUrl: (objectId: string, attachmentId: string) =>
    `${BASE_URL}/objects/${objectId}/attachments/${attachmentId}/download`,

  deleteAttachment: (objectId: string, attachmentId: string) =>
    request<void>(`/objects/${objectId}/attachments/${attachmentId}`, { method: "DELETE" }),

  verifyAttachment: (objectId: string, attachmentId: string) =>
    request<{ attachment_id: string; file_name: string; expected_sha256: string | null; actual_sha256: string; valid: boolean }>(
      `/objects/${objectId}/attachments/${attachmentId}/verify`,
    ),

  // --- Webhooks ---
  listWebhooks: (moduleId: string) =>
    request<Webhook[]>(`/modules/${moduleId}/webhooks`),

  createWebhook: (moduleId: string, data: { name: string; url: string; secret?: string; events?: string; active?: boolean }) =>
    request<Webhook>(`/modules/${moduleId}/webhooks`, { method: "POST", body: JSON.stringify(data) }),

  getWebhook: (moduleId: string, id: string) =>
    request<Webhook>(`/modules/${moduleId}/webhooks/${id}`),

  updateWebhook: (moduleId: string, id: string, data: { name?: string; url?: string; secret?: string; events?: string; active?: boolean }) =>
    request<Webhook>(`/modules/${moduleId}/webhooks/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteWebhook: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/webhooks/${id}`, { method: "DELETE" }),

  // --- Lifecycle Models ---
  listLifecycleModels: (moduleId: string) =>
    request<LifecycleModel[]>(`/modules/${moduleId}/lifecycle-models`),

  createLifecycleModel: (moduleId: string, data: {
    name: string;
    description?: string;
    initial_state?: string;
    states?: Array<{ name: string; color?: string; description?: string }>;
    transitions?: Array<{ from: string; to: string }>;
  }) =>
    request<LifecycleModel>(`/modules/${moduleId}/lifecycle-models`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getLifecycleModel: (moduleId: string, id: string) =>
    request<LifecycleModel>(`/modules/${moduleId}/lifecycle-models/${id}`),

  updateLifecycleModel: (moduleId: string, id: string, data: {
    name?: string;
    description?: string;
    initial_state?: string;
    states?: Array<{ name: string; color?: string; description?: string }>;
    transitions?: Array<{ from: string; to: string }>;
  }) =>
    request<LifecycleModel>(`/modules/${moduleId}/lifecycle-models/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteLifecycleModel: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/lifecycle-models/${id}`, { method: "DELETE" }),

  // --- XLSX Import ---
  importXlsx: async (moduleId: string, file: File): Promise<{ objects_created: number; objects_updated: number }> => {
    const buffer = await file.arrayBuffer();
    const token = localStorage.getItem("token");
    const headers: Record<string, string> = { "Content-Type": "application/octet-stream" };
    if (token) headers["Authorization"] = `Bearer ${token}`;
    const res = await fetch(`${BASE_URL}/modules/${moduleId}/import/xlsx`, {
      method: "POST",
      headers,
      body: buffer,
    });
    if (!res.ok) {
      const body = await res.json().catch(() => null);
      throw new Error(body?.error?.message ?? `XLSX import failed: ${res.status}`);
    }
    return res.json();
  },

  // --- Global Search ---
  searchGlobal: (query: string, limit?: number) => {
    const params = new URLSearchParams();
    params.set("q", query);
    if (limit != null) params.set("limit", String(limit));
    return request<{ items: Array<ReqObject & { module_name: string }> }>(
      `/search?${params.toString()}`,
    );
  },

  // --- Test Cases ---
  listTestCases: (moduleId: string) =>
    request<TestCase[]>(`/modules/${moduleId}/test-cases`),

  createTestCase: (moduleId: string, data: {
    name: string;
    description?: string;
    preconditions?: string;
    expected_result?: string;
    test_type?: string;
    priority?: string;
    status?: string;
    requirement_ids?: string[];
  }) =>
    request<TestCase>(`/modules/${moduleId}/test-cases`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getTestCase: (moduleId: string, id: string) =>
    request<TestCase>(`/modules/${moduleId}/test-cases/${id}`),

  updateTestCase: (moduleId: string, id: string, data: {
    name?: string;
    description?: string;
    preconditions?: string;
    expected_result?: string;
    test_type?: string;
    priority?: string;
    status?: string;
    requirement_ids?: string[];
  }) =>
    request<TestCase>(`/modules/${moduleId}/test-cases/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteTestCase: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/test-cases/${id}`, { method: "DELETE" }),

  // --- Test Executions ---
  listTestExecutions: (testCaseId: string) =>
    request<TestExecution[]>(`/test-cases/${testCaseId}/executions`),

  createTestExecution: (testCaseId: string, data: {
    status?: string;
    executor?: string;
    executed_at?: string;
    duration_ms?: number;
    evidence?: string;
    environment?: string;
  }) =>
    request<TestExecution>(`/test-cases/${testCaseId}/executions`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  updateTestExecution: (testCaseId: string, id: string, data: {
    status?: string;
    executor?: string;
    evidence?: string;
    environment?: string;
  }) =>
    request<TestExecution>(`/test-cases/${testCaseId}/executions/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteTestExecution: (testCaseId: string, id: string) =>
    request<void>(`/test-cases/${testCaseId}/executions/${id}`, { method: "DELETE" }),

  // --- Test Coverage & Dashboard ---
  getTestCoverage: (moduleId: string) =>
    request<TestCoverageResponse>(`/modules/${moduleId}/test-coverage`),

  getTestDashboard: (moduleId: string) =>
    request<TestDashboardSummary>(`/modules/${moduleId}/test-dashboard`),

  // --- Allowed Values (Dependent Enums) ---
  getAllowedValues: (moduleId: string, attrDefId: string, parentValue: string) =>
    request<string[]>(
      `/modules/${moduleId}/attribute-definitions/${attrDefId}/allowed-values?parent_value=${encodeURIComponent(parentValue)}`,
    ),

  // --- Review Comments ---
  listReviewComments: (packageId: string) =>
    request<PaginatedResponse<ReviewComment>>(`/review-packages/${packageId}/comments`),

  createReviewComment: (packageId: string, data: { body: string; author_id?: string }) =>
    request<ReviewComment>(`/review-packages/${packageId}/comments`, { method: "POST", body: JSON.stringify(data) }),

  updateReviewComment: (packageId: string, id: string, data: { body?: string }) =>
    request<ReviewComment>(`/review-packages/${packageId}/comments/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteReviewComment: (packageId: string, id: string) =>
    request<void>(`/review-packages/${packageId}/comments/${id}`, { method: "DELETE" }),

  // --- Voting Summary ---
  getVotingSummary: (moduleId: string) =>
    request<VotingSummary[]>(`/modules/${moduleId}/review-packages/voting-summary`),

  // --- Placeholder Operations ---
  syncPlaceholder: (moduleId: string, objectId: string) =>
    request<ReqObject>(`/modules/${moduleId}/objects/${objectId}/sync`, { method: "POST" }),

  breakPlaceholderLink: (moduleId: string, objectId: string) =>
    request<ReqObject>(`/modules/${moduleId}/objects/${objectId}/break-link`, { method: "POST" }),

  syncAllPlaceholders: (moduleId: string) =>
    request<{ synced: number }>(`/modules/${moduleId}/sync-placeholders`, { method: "POST" }),

  // --- Diagrams ---
  listDiagrams: (moduleId: string) =>
    request<PaginatedResponse<Diagram>>(`/modules/${moduleId}/diagrams`),

  createDiagram: (moduleId: string, data: {
    name: string;
    description?: string;
    diagram_type?: string;
    source_code?: string;
    linked_object_ids?: string[];
  }) =>
    request<Diagram>(`/modules/${moduleId}/diagrams`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getDiagram: (moduleId: string, id: string) =>
    request<Diagram>(`/modules/${moduleId}/diagrams/${id}`),

  updateDiagram: (moduleId: string, id: string, data: {
    name?: string;
    description?: string;
    diagram_type?: string;
    source_code?: string;
    linked_object_ids?: string[];
  }) =>
    request<Diagram>(`/modules/${moduleId}/diagrams/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteDiagram: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/diagrams/${id}`, { method: "DELETE" }),

  // --- Script Executions ---
  listScriptExecutions: (moduleId: string, scriptId: string) =>
    request<PaginatedResponse<ScriptExecution>>(`/modules/${moduleId}/scripts/${scriptId}/executions`),

  // --- Audit Log ---
  listAuditLog: (filters?: { user_id?: string; entity_type?: string; action?: string }) => {
    const params = new URLSearchParams();
    if (filters?.user_id) params.set("user_id", filters.user_id);
    if (filters?.entity_type) params.set("entity_type", filters.entity_type);
    if (filters?.action) params.set("action", filters.action);
    const qs = params.toString();
    return request<PaginatedResponse<AuditLogEntry>>(`/audit-log${qs ? `?${qs}` : ""}`);
  },

  // --- Notifications ---
  listNotifications: (filters?: { offset?: number; limit?: number; unread_only?: boolean }) => {
    const params = new URLSearchParams();
    if (filters?.offset != null) params.set("offset", String(filters.offset));
    if (filters?.limit != null) params.set("limit", String(filters.limit));
    if (filters?.unread_only) params.set("unread_only", "true");
    const qs = params.toString();
    return request<PaginatedResponse<Notification>>(`/notifications${qs ? `?${qs}` : ""}`);
  },

  getUnreadCount: () =>
    request<{ count: number }>("/notifications/unread-count"),

  markNotificationRead: (id: string) =>
    request<Notification>(`/notifications/${id}/read`, { method: "POST" }),

  markAllNotificationsRead: () =>
    request<{ updated: number }>("/notifications/read-all", { method: "POST" }),

  // --- E-Signatures ---
  createSignature: (data: { entity_type: string; entity_id: string; password: string; meaning: string }) =>
    request<ESignature>("/e-signatures", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  listSignatures: (entityType: string, entityId: string) =>
    request<ESignature[]>(`/e-signatures/entity/${entityType}/${entityId}`),

  // --- Review Package Transition ---
  transitionReviewPackage: (moduleId: string, id: string, data: { status: string; password?: string; meaning?: string }) =>
    request<ReviewPackage>(`/modules/${moduleId}/review-packages/${id}/transition`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  // --- Dashboards ---
  listDashboards: (workspaceId: string) =>
    request<Dashboard[]>(`/workspaces/${workspaceId}/dashboards`),

  createDashboard: (workspaceId: string, data: { name: string; description?: string }) =>
    request<Dashboard>(`/workspaces/${workspaceId}/dashboards`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getDashboard: (workspaceId: string, id: string) =>
    request<Dashboard>(`/workspaces/${workspaceId}/dashboards/${id}`),

  updateDashboard: (workspaceId: string, id: string, data: { name?: string; description?: string }) =>
    request<Dashboard>(`/workspaces/${workspaceId}/dashboards/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteDashboard: (workspaceId: string, id: string) =>
    request<void>(`/workspaces/${workspaceId}/dashboards/${id}`, { method: "DELETE" }),

  // --- Dashboard Widgets ---
  listWidgets: (dashboardId: string) =>
    request<DashboardWidget[]>(`/dashboards/${dashboardId}/widgets`),

  createWidget: (dashboardId: string, data: {
    widget_type: string;
    title: string;
    config?: Record<string, unknown>;
    position_x?: number;
    position_y?: number;
    width?: number;
    height?: number;
  }) =>
    request<DashboardWidget>(`/dashboards/${dashboardId}/widgets`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getWidget: (dashboardId: string, id: string) =>
    request<DashboardWidget>(`/dashboards/${dashboardId}/widgets/${id}`),

  updateWidget: (dashboardId: string, id: string, data: {
    widget_type?: string;
    title?: string;
    config?: Record<string, unknown>;
    position_x?: number;
    position_y?: number;
    width?: number;
    height?: number;
  }) =>
    request<DashboardWidget>(`/dashboards/${dashboardId}/widgets/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteWidget: (dashboardId: string, id: string) =>
    request<void>(`/dashboards/${dashboardId}/widgets/${id}`, { method: "DELETE" }),

  getWidgetData: (dashboardId: string, widgetId: string) =>
    request<WidgetDataEntry[]>(`/dashboards/${dashboardId}/widgets/${widgetId}/data`),

  exportDashboardPdfUrl: (dashboardId: string) =>
    `${BASE_URL}/dashboards/${dashboardId}/export/pdf`,

  // --- Project Templates ---
  listProjectTemplates: () =>
    request<ProjectTemplate[]>("/project-templates"),

  createProjectTemplate: (data: {
    name: string;
    description?: string;
    standard?: string;
    version?: string;
    template_data: unknown;
  }) =>
    request<ProjectTemplate>("/project-templates", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getProjectTemplate: (id: string) =>
    request<ProjectTemplate>(`/project-templates/${id}`),

  updateProjectTemplate: (id: string, data: {
    name?: string;
    description?: string;
    standard?: string;
    version?: string;
    template_data?: unknown;
  }) =>
    request<ProjectTemplate>(`/project-templates/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteProjectTemplate: (id: string) =>
    request<void>(`/project-templates/${id}`, { method: "DELETE" }),

  instantiateTemplate: (id: string, data: {
    workspace_id: string;
    project_name: string;
    project_description?: string;
    include_seed_objects?: boolean;
  }) =>
    request<InstantiateResult>(`/project-templates/${id}/instantiate`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  // --- DOCX Import ---
  previewDocx: async (moduleId: string, file: File): Promise<DocxPreviewResult> => {
    const buffer = await file.arrayBuffer();
    const token = localStorage.getItem("token");
    const headers: Record<string, string> = { "Content-Type": "application/octet-stream" };
    if (token) headers["Authorization"] = `Bearer ${token}`;
    const res = await fetch(`${BASE_URL}/modules/${moduleId}/import/docx/preview`, {
      method: "POST",
      headers,
      body: buffer,
    });
    if (!res.ok) {
      const body = await res.json().catch(() => null);
      throw new Error(body?.error?.message ?? `Preview failed: ${res.status}`);
    }
    return res.json();
  },

  importDocx: async (moduleId: string, file: File, mappings: Array<{ style_id: string; classification: string; is_heading: boolean }>): Promise<DocxImportResult> => {
    const formData = new FormData();
    formData.append("file", file);
    formData.append("mapping", JSON.stringify({ style_mappings: mappings }));
    const token = localStorage.getItem("token");
    const headers: Record<string, string> = {};
    if (token) headers["Authorization"] = `Bearer ${token}`;
    const res = await fetch(`${BASE_URL}/modules/${moduleId}/import/docx`, {
      method: "POST",
      headers,
      body: formData,
    });
    if (!res.ok) {
      const body = await res.json().catch(() => null);
      throw new Error(body?.error?.message ?? `Import failed: ${res.status}`);
    }
    return res.json();
  },
};
