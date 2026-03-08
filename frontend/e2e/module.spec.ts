import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, waitForApp, navigateToModule, scaffold,
  createWorkspace, createProject, createModule, createAttrDef,
  apiGet, apiPost, apiPatch, apiDelete, API, auth,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

test.describe("Module CRUD", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "modcrud");

    const got = await apiGet(request, `/modules/${mod.id}`);
    expect(got.name).toBe(mod.name);

    const newName = `Mod-updated-${uid()}`;
    const updated = await apiPatch(request, `/modules/${mod.id}`, { name: newName });
    expect(updated.name).toBe(newName);

    await apiDelete(request, `/modules/${mod.id}`);
    const res = await request.get(`${API}/modules/${mod.id}`, { headers: auth() });
    expect(res.status()).toBe(404);
  });

  test("UI: selecting a module shows all 14 tabs", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "modtab");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    for (const tab of [
      "Objects", "Document", "Links", "Baselines", "Attributes",
      "Scripts", "Validation", "Types", "Reviews", "Proposals",
      "Lifecycle", "Tests", "Diagrams", "Settings",
    ]) {
      await expect(page.getByRole("button", { name: tab })).toBeVisible();
    }
  });

  test("UI: rename module from sidebar", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "modrn");
    const newName = `Mod-renamed-${uid()}`;
    await waitForApp(page);
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(mod.name).hover();
    await page.waitForTimeout(200);
    await page.getByTitle("Rename").first().click();
    await page.waitForTimeout(200);
    const input = page.locator("input:focus");
    await expect(input).toBeVisible({ timeout: 3000 });
    await input.fill(newName);
    await input.press("Enter");
    await expect(page.getByText(newName)).toBeVisible({ timeout: 3000 });
  });

  test("UI: delete module from sidebar", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "moddel");
    await waitForApp(page);
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(mod.name).hover();
    await page.waitForTimeout(200);
    await page.getByTitle("Delete").first().click();
    await page.waitForTimeout(500);
    await expect(page.getByText(mod.name)).not.toBeVisible();
  });

  test("API: create module from template", async ({ request }) => {
    const ws = await createWorkspace(request, `WS-mft-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-mft-${uid()}`);
    const srcMod = await createModule(request, proj.id, `Src-${uid()}`);
    await createAttrDef(request, srcMod.id, `attr-${uid()}`, "string");

    const res = await request.post(`${API}/modules/from-template`, {
      data: {
        name: `Clone-${uid()}`,
        project_id: proj.id,
        template_module_id: srcMod.id,
        copy_objects: false,
      },
      headers: auth(),
    });
    expect(res.status()).toBe(201);
    const cloned = await res.json();
    expect(cloned.id).toBeTruthy();
    expect(cloned.project_id).toBe(proj.id);
  });

  test("UI: create module from template", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-tpl-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-tpl-${uid()}`);
    const tplMod = await createModule(request, proj.id, `Template-${uid()}`);
    await createAttrDef(request, tplMod.id, "priority", "string");

    await waitForApp(page);
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).hover();
    await page.getByTitle("Add module").click();
    await page.waitForTimeout(300);

    const newModName = `FromTpl-${uid()}`;
    await page.getByPlaceholder("Module name").fill(newModName);
    const templateSelect = page.locator("select").first();
    await templateSelect.selectOption({ label: `Template: ${tplMod.name}` });
    await page.getByPlaceholder("Module name").press("Enter");
    await page.waitForTimeout(500);
    await expect(page.getByText(newModName)).toBeVisible({ timeout: 3000 });
  });
});

test.describe("Tabs & Settings", () => {
  test("Objects tab is default with search bar", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "tab");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await expect(page.getByPlaceholder("Full-text search...")).toBeVisible();
  });

  test("switching to Links tab shows link form", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "tablink");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Links" }).click();
    await expect(page.getByRole("button", { name: "Create Link" })).toBeVisible();
  });

  test("switching to Baselines tab shows baseline form", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "tabbl");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Baselines" }).click();
    await expect(page.getByPlaceholder("Baseline name")).toBeVisible();
  });

  test("switching to Attributes tab shows attr form", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "tabattr");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Attributes" }).click();
    await expect(page.getByPlaceholder("Attribute name")).toBeVisible();
  });

  test("settings form is visible and saveable", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "setui");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Settings" }).click();
    await page.waitForTimeout(300);

    await expect(page.getByPlaceholder("e.g. REQ")).toBeVisible();
    await page.getByPlaceholder("e.g. REQ").fill("SPEC");
    await page.getByRole("button", { name: /Save Settings/ }).click();
    await expect(page.getByText("Saved")).toBeVisible({ timeout: 5000 });
  });
});
