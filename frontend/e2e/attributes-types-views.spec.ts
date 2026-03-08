import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, navigateToModule, scaffold,
  createObject, createAttrDef,
  apiGet, apiPost, apiPatch, apiDelete,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

// ---------------------------------------------------------------------------
// Attribute Definitions
// ---------------------------------------------------------------------------

test.describe("Attribute Definitions", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "attrcrud");
    const attr = await createAttrDef(request, mod.id, `attr-${uid()}`, "string");

    const got = await apiGet(request, `/modules/${mod.id}/attribute-definitions/${attr.id}`);
    expect(got.name).toBeTruthy();

    const updated = await apiPatch(
      request,
      `/modules/${mod.id}/attribute-definitions/${attr.id}`,
      { name: `attr-updated-${uid()}` },
    );
    expect(updated.name).toContain("updated");

    await apiDelete(request, `/modules/${mod.id}/attribute-definitions/${attr.id}`);
  });

  test("API: enum attribute with allowed values", async ({ request }) => {
    const { mod } = await scaffold(request, "attreval");
    const attr = await apiPost(request, `/modules/${mod.id}/attribute-definitions`, {
      name: `status-${uid()}`,
      data_type: "enum",
      enum_values: ["open", "closed", "pending"],
    });
    expect(attr.id).toBeTruthy();

    const got = await apiGet(request, `/modules/${mod.id}/attribute-definitions/${attr.id}`);
    expect(got.data_type).toBe("enum");

    await apiDelete(request, `/modules/${mod.id}/attribute-definitions/${attr.id}`);
  });

  test("API: list all attribute definitions for module", async ({ request }) => {
    const { mod } = await scaffold(request, "attrlist");
    await createAttrDef(request, mod.id, `a1-${uid()}`, "string");
    await createAttrDef(request, mod.id, `a2-${uid()}`, "integer");

    const list = await apiGet(request, `/modules/${mod.id}/attribute-definitions`);
    expect(list.items.length).toBeGreaterThanOrEqual(2);
  });

  test("UI: create and delete attribute definition", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "attrui");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Attributes" }).click();
    await page.waitForTimeout(300);

    const attrName = `attr-${uid()}`;
    await page.getByPlaceholder("Attribute name").fill(attrName);
    await page.getByRole("button", { name: /Create|Add/ }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(attrName)).toBeVisible({ timeout: 3000 });

    await page.getByRole("button", { name: /Delete/ }).first().click();
    await page.waitForTimeout(500);
    await expect(page.getByText(attrName)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Object Types
// ---------------------------------------------------------------------------

test.describe("Object Types", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "otcrud");
    const typeName = `Type-${uid()}`;
    const ot = await apiPost(request, "/object-types", { name: typeName, module_id: mod.id });
    expect(ot.id).toBeTruthy();

    const got = await apiGet(request, `/object-types/${ot.id}`);
    expect(got.name).toBe(typeName);

    const updated = await apiPatch(request, `/object-types/${ot.id}`, {
      name: `Type-upd-${uid()}`,
    });
    expect(updated.name).toContain("upd");

    await apiDelete(request, `/object-types/${ot.id}`);
  });

  test("UI: create and delete an object type", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "typeui");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Types" }).click();
    await page.waitForTimeout(300);

    const typeName = `Type-${uid()}`;
    await page.getByPlaceholder("Type name").fill(typeName);
    await page.getByRole("button", { name: /^Add$/ }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(typeName)).toBeVisible({ timeout: 3000 });

    await page.getByRole("button", { name: /Delete/ }).first().click();
    await page.waitForTimeout(500);
    await expect(page.getByText(typeName)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

test.describe("Views", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "viewcrud");
    const view = await apiPost(request, `/modules/${mod.id}/views`, {
      name: `View-${uid()}`,
      column_config: { columns: ["heading", "body"] },
    });
    expect(view.id).toBeTruthy();

    // Get by ID
    const got = await apiGet(request, `/modules/${mod.id}/views/${view.id}`);
    expect(got.id).toBe(view.id);
    expect(got.name).toBe(view.name);

    // Update
    const updated = await apiPatch(request, `/modules/${mod.id}/views/${view.id}`, {
      name: `View-upd-${uid()}`,
      column_config: { columns: ["heading", "body", "position"] },
      is_default: true,
    });
    expect(updated.name).toContain("upd");

    // List
    const list = await apiGet(request, `/modules/${mod.id}/views`);
    expect(list.items.length).toBeGreaterThan(0);

    await apiDelete(request, `/modules/${mod.id}/views/${view.id}`);
  });

  test("UI: save and delete a view", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "viewui");
    await createObject(request, mod.id, `VIEW-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(500);

    await page.getByRole("button", { name: /Save As/ }).click();
    await page.waitForTimeout(300);
    const viewName = `View-${uid()}`;
    await page.getByPlaceholder("View name").fill(viewName);
    await page.getByRole("button", { name: /^Save$/ }).click();
    await page.waitForTimeout(500);

    const viewSelect = page.locator("select").first();
    await expect(viewSelect).toContainText(viewName);
    await viewSelect.selectOption({ label: viewName });
    await page.waitForTimeout(300);
    await page.getByRole("button", { name: /Delete/ }).click();
    await page.waitForTimeout(300);
    await expect(viewSelect).not.toContainText(viewName);
  });
});
