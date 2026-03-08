import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, waitForApp,
  createWorkspace, createProject,
  apiGet, apiPatch, apiDelete, auth, API,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

// ---------------------------------------------------------------------------
// Workspace
// ---------------------------------------------------------------------------

test.describe("Workspace CRUD", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const ws = await createWorkspace(request, `WS-crud-${uid()}`);
    expect(ws.id).toBeTruthy();

    const got = await apiGet(request, `/workspaces/${ws.id}`);
    expect(got.name).toBe(ws.name);

    const newName = `WS-updated-${uid()}`;
    const updated = await apiPatch(request, `/workspaces/${ws.id}`, { name: newName });
    expect(updated.name).toBe(newName);

    await apiDelete(request, `/workspaces/${ws.id}`);
    const res = await request.get(`${API}/workspaces/${ws.id}`, { headers: auth() });
    expect(res.status()).toBe(404);
  });

  test("API: list workspaces", async ({ request }) => {
    await createWorkspace(request, `WS-list-${uid()}`);
    const body = await apiGet(request, "/workspaces");
    expect(body.items.length).toBeGreaterThan(0);
  });

  test("UI: create workspace from sidebar", async ({ page }) => {
    await waitForApp(page);
    const name = `WS-${uid()}`;
    await page.locator("div").filter({ hasText: /^Navigation/ }).getByTitle("New workspace").click();
    await page.getByPlaceholder("Workspace name").fill(name);
    await page.getByPlaceholder("Workspace name").press("Enter");
    await expect(page.getByText(name)).toBeVisible();
  });

  test("UI: select workspace shows detail", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-detail-${uid()}`);
    await waitForApp(page);
    await page.getByText(ws.name).click();
    await expect(page.locator("main h1")).toContainText(ws.name);
  });

  test("UI: rename workspace from sidebar", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-rename-${uid()}`);
    const newName = `WS-renamed-${uid()}`;
    await waitForApp(page);
    await page.getByText(ws.name, { exact: true }).hover();
    await page.waitForTimeout(200);
    await page.getByTitle("Rename").first().click();
    await page.waitForTimeout(200);
    const input = page.locator("input:focus");
    await expect(input).toBeVisible({ timeout: 3000 });
    await input.fill(newName);
    await input.press("Enter");
    await expect(page.getByText(newName)).toBeVisible({ timeout: 3000 });
  });

  test("UI: delete workspace from sidebar", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-delete-${uid()}`);
    await waitForApp(page);
    await page.getByText(ws.name).hover();
    await page.waitForTimeout(200);
    await page.getByTitle("Delete").first().click();
    await page.waitForTimeout(500);
    await expect(page.getByText(ws.name)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Project
// ---------------------------------------------------------------------------

test.describe("Project CRUD", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const ws = await createWorkspace(request, `WS-projcrud-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-crud-${uid()}`);
    expect(proj.id).toBeTruthy();

    const got = await apiGet(request, `/workspaces/${ws.id}/projects/${proj.id}`);
    expect(got.name).toBe(proj.name);

    const newName = `Proj-updated-${uid()}`;
    const updated = await apiPatch(request, `/workspaces/${ws.id}/projects/${proj.id}`, {
      name: newName,
    });
    expect(updated.name).toBe(newName);

    await apiDelete(request, `/workspaces/${ws.id}/projects/${proj.id}`);
    const res = await request.get(`${API}/workspaces/${ws.id}/projects/${proj.id}`, {
      headers: auth(),
    });
    expect(res.status()).toBe(404);
  });

  test("UI: create project under workspace", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-proj-${uid()}`);
    const projName = `Proj-${uid()}`;
    await waitForApp(page);
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(ws.name).hover();
    await page.getByTitle("Add project").click();
    await page.getByPlaceholder("Project name").fill(projName);
    await page.getByPlaceholder("Project name").press("Enter");
    await expect(page.getByText(projName)).toBeVisible();
  });

  test("UI: rename project from sidebar", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-prn-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-rn-${uid()}`);
    const newName = `Proj-renamed-${uid()}`;
    await waitForApp(page);
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).hover();
    await page.waitForTimeout(200);
    await page.getByTitle("Rename").first().click();
    await page.waitForTimeout(200);
    const input = page.locator("input:focus");
    await expect(input).toBeVisible({ timeout: 3000 });
    await input.fill(newName);
    await input.press("Enter");
    await expect(page.getByText(newName)).toBeVisible({ timeout: 3000 });
  });

  test("UI: delete project from sidebar", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-pdel-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-del-${uid()}`);
    await waitForApp(page);
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).hover();
    await page.waitForTimeout(200);
    await page.getByTitle("Delete").first().click();
    await page.waitForTimeout(500);
    await expect(page.getByText(proj.name)).not.toBeVisible();
  });
});
