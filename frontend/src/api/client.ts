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
  created_at: string;
  updated_at: string;
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

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...init,
  });

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
    data: { heading?: string; body?: string; position?: number; object_type_id?: string; classification?: string },
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
      attributes?: Record<string, unknown>;
      reviewed?: boolean;
    },
  ) =>
    request<ReqObject>(`/modules/${moduleId}/objects/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),

  deleteObject: (moduleId: string, id: string) =>
    request<void>(`/modules/${moduleId}/objects/${id}`, { method: "DELETE" }),

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
    data: { name: string; data_type: string; default_value?: string; enum_values?: string[] },
  ) =>
    request<AttributeDefinition>(`/modules/${moduleId}/attribute-definitions`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  updateAttributeDefinition: (
    moduleId: string,
    id: string,
    data: { name?: string; data_type?: string; default_value?: string; enum_values?: string[] },
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
  }) =>
    request<Module>("/modules/from-template", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  // --- Coverage ---
  getCoverage: (moduleId: string) =>
    request<CoverageResponse>(`/modules/${moduleId}/coverage`),
};
