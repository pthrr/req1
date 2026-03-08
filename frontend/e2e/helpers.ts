import { expect, type Page, type APIRequestContext } from "@playwright/test";

export const API = "http://localhost:8080/api/v1";
let TOKEN = "";

/** Call in test.beforeAll — idempotent, only fetches once. */
export async function setupAuth() {
  if (TOKEN) return;
  const res = await fetch(`${API}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email: "admin@localhost", password: "admin" }),
  });
  expect(res.ok).toBeTruthy();
  const data = await res.json();
  TOKEN = data.token;
}

export function auth() {
  return { Authorization: `Bearer ${TOKEN}` };
}

export function getToken() {
  return TOKEN;
}

export function uid() {
  return Math.random().toString(36).slice(2, 8);
}

// ---------------------------------------------------------------------------
// Generic API helpers
// ---------------------------------------------------------------------------

export async function apiPost(request: APIRequestContext, path: string, data: object) {
  const res = await request.post(`${API}${path}`, { data, headers: auth() });
  expect(res.ok(), `POST ${path} failed: ${res.status()}`).toBeTruthy();
  return res.json();
}

export async function apiGet(request: APIRequestContext, path: string) {
  const res = await request.get(`${API}${path}`, { headers: auth() });
  expect(res.ok(), `GET ${path} failed: ${res.status()}`).toBeTruthy();
  return res.json();
}

export async function apiPatch(request: APIRequestContext, path: string, data: object) {
  const res = await request.patch(`${API}${path}`, { data, headers: auth() });
  expect(res.ok(), `PATCH ${path} failed: ${res.status()}`).toBeTruthy();
  return res.json();
}

export async function apiDelete(request: APIRequestContext, path: string) {
  const res = await request.delete(`${API}${path}`, { headers: auth() });
  expect(res.ok(), `DELETE ${path} failed: ${res.status()}`).toBeTruthy();
}

// ---------------------------------------------------------------------------
// Domain factories
// ---------------------------------------------------------------------------

export async function createWorkspace(request: APIRequestContext, name: string) {
  return apiPost(request, "/workspaces", { name }) as Promise<{ id: string; name: string }>;
}

export async function createProject(request: APIRequestContext, wsId: string, name: string) {
  return apiPost(request, `/workspaces/${wsId}/projects`, { name }) as Promise<{
    id: string;
    name: string;
  }>;
}

export async function createModule(request: APIRequestContext, projectId: string, name: string) {
  return apiPost(request, "/modules", { name, project_id: projectId }) as Promise<{
    id: string;
    name: string;
  }>;
}

export async function createObject(
  request: APIRequestContext,
  moduleId: string,
  heading: string,
  body?: string,
) {
  return apiPost(request, `/modules/${moduleId}/objects`, { heading, body }) as Promise<{
    id: string;
  }>;
}

export async function createLinkType(request: APIRequestContext, name: string) {
  return apiPost(request, "/link-types", { name }) as Promise<{ id: string; name: string }>;
}

export async function createLink(
  request: APIRequestContext,
  sourceId: string,
  targetId: string,
  linkTypeId: string,
) {
  return apiPost(request, "/links", {
    source_object_id: sourceId,
    target_object_id: targetId,
    link_type_id: linkTypeId,
  }) as Promise<{ id: string }>;
}

export async function createAttrDef(
  request: APIRequestContext,
  moduleId: string,
  name: string,
  dataType: string,
) {
  return apiPost(request, `/modules/${moduleId}/attribute-definitions`, {
    name,
    data_type: dataType,
  }) as Promise<{ id: string }>;
}

export async function createReviewPackage(
  request: APIRequestContext,
  moduleId: string,
  name: string,
) {
  return apiPost(request, `/modules/${moduleId}/review-packages`, { name }) as Promise<{
    id: string;
    name: string;
    status: string;
  }>;
}

export async function createChangeProposal(
  request: APIRequestContext,
  moduleId: string,
  title: string,
) {
  return apiPost(request, `/modules/${moduleId}/change-proposals`, { title }) as Promise<{
    id: string;
    title: string;
    status: string;
  }>;
}

export async function createBaselineSet(
  request: APIRequestContext,
  name: string,
  version: string,
) {
  return apiPost(request, "/baseline-sets", { name, version }) as Promise<{
    id: string;
    name: string;
    version: string;
  }>;
}

// ---------------------------------------------------------------------------
// Navigation helpers
// ---------------------------------------------------------------------------

/** Wait for the app to load (handles dev auto-login redirect). */
export async function waitForApp(page: Page) {
  await page.goto("/");
  await expect(page.getByText("Navigation")).toBeVisible({ timeout: 15000 });
}

/** Navigate from / to a module's detail page via sidebar clicks. */
export async function navigateToModule(
  page: Page,
  wsName: string,
  projName: string,
  modName: string,
) {
  await waitForApp(page);
  await page.getByText(wsName).locator("..").locator("span").first().click();
  await page.waitForTimeout(300);
  await page.getByText(projName).locator("..").locator("span").first().click();
  await page.waitForTimeout(300);
  await page.getByText(modName).click();
  await page.waitForTimeout(500);
}

/** Scaffold workspace → project → module via API. */
export async function scaffold(request: APIRequestContext, prefix: string) {
  const ws = await createWorkspace(request, `WS-${prefix}-${uid()}`);
  const proj = await createProject(request, ws.id, `Proj-${prefix}-${uid()}`);
  const mod = await createModule(request, proj.id, `Mod-${prefix}-${uid()}`);
  return { ws, proj, mod };
}

/**
 * Click an action button on the first ag-grid row. If the click opens the
 * history modal instead (ag-grid row click race), close it and retry.
 */
export async function clickRowAction(page: Page, buttonName: string) {
  const btn = page.getByRole("button", { name: buttonName, exact: true }).first();
  await btn.click();
  await page.waitForTimeout(500);
  if (await page.getByText(/History for/).isVisible()) {
    await page.getByRole("button", { name: "Close" }).click();
    await page.waitForTimeout(300);
    await btn.click();
    await page.waitForTimeout(500);
  }
}
