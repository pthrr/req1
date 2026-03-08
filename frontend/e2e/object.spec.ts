import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, navigateToModule, scaffold,
  createObject, apiGet, apiPatch, apiDelete, apiPost, clickRowAction,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

test.describe("Object CRUD", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "objcrud");
    const heading = `OBJ-${uid()}`;
    const obj = await createObject(request, mod.id, heading, "Initial body");

    const got = await apiGet(request, `/modules/${mod.id}/objects/${obj.id}`);
    expect(got.heading).toBe(heading);

    const updated = await apiPatch(request, `/modules/${mod.id}/objects/${obj.id}`, {
      heading: "Updated heading",
      body: "Updated body",
    });
    expect(updated.heading).toBe("Updated heading");

    await apiDelete(request, `/modules/${mod.id}/objects/${obj.id}`);
  });

  test("API: list objects in module", async ({ request }) => {
    const { mod } = await scaffold(request, "objlist");
    await createObject(request, mod.id, `A-${uid()}`);
    await createObject(request, mod.id, `B-${uid()}`);
    const body = await apiGet(request, `/modules/${mod.id}/objects`);
    expect(body.items.length).toBe(2);
  });

  test("API: object history", async ({ request }) => {
    const { mod } = await scaffold(request, "objhist");
    const obj = await createObject(request, mod.id, `HIST-${uid()}`);
    await apiPatch(request, `/modules/${mod.id}/objects/${obj.id}`, { body: "v2" });
    const hist = await apiGet(request, `/modules/${mod.id}/objects/${obj.id}/history`);
    expect(hist.items.length).toBeGreaterThanOrEqual(1);
  });

  test("API: move object", async ({ request }) => {
    const { mod } = await scaffold(request, "objmove");
    // Create objects with explicit positions to ensure deterministic ordering
    const a = await apiPost(request, `/modules/${mod.id}/objects`, {
      heading: `MOVE-A-${uid()}`, position: 0,
    }) as { id: string };
    await apiPost(request, `/modules/${mod.id}/objects`, {
      heading: `MOVE-B-${uid()}`, position: 1,
    });
    const moved = await apiPost(request, `/modules/${mod.id}/objects/${a.id}/move`, {
      action: "down",
    });
    expect(moved).toBeTruthy();
  });

  test("UI: create object via form", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "objadd");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    const heading = `REQ-${uid()}`;
    await page.getByPlaceholder("Heading (e.g. REQ-001)").fill(heading);
    await page.getByPlaceholder("Body", { exact: true }).fill("Test body text");
    await page.getByRole("button", { name: "Add" }).click();
    await expect(page.locator(".ag-root-wrapper")).toContainText(heading, { timeout: 5000 });
  });

  test("UI: delete object via grid button", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "objdel");
    const heading = `DEL-${uid()}`;
    await createObject(request, mod.id, heading);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    page.on("dialog", (dialog) => dialog.accept());
    await page.getByRole("button", { name: "Del" }).first().click();
    await page.waitForTimeout(500);
    await expect(page.locator(".ag-root-wrapper")).not.toContainText(heading);
  });

  test("UI: edit object via detail panel", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "objedit");
    await createObject(request, mod.id, `EDIT-${uid()}`, "Original body");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await clickRowAction(page, "Edit");
    await expect(page.getByText(/^Object:/)).toBeVisible({ timeout: 5000 });
    await page.getByRole("button", { name: "Save", exact: true }).first().click();
    await page.waitForTimeout(500);
    await page.getByRole("button", { name: "Close" }).click();
    await expect(page.getByText(/^Object:/)).not.toBeVisible();
  });
});

test.describe("Object History Modal", () => {
  test("clicking a row opens history modal", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "hist");
    await createObject(request, mod.id, `REQ-HIST-${uid()}`, "History test body");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);
    await page.locator(".ag-row").first().click();
    await expect(page.getByText("History for")).toBeVisible({ timeout: 5000 });
    await page.getByRole("button", { name: "Close" }).click();
    await expect(page.getByText("History for")).not.toBeVisible();
  });
});

test.describe("Split View", () => {
  test("toggle split view shows tree + detail panel", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "sv");
    const heading = `SV-${uid()}`;
    await createObject(request, mod.id, heading);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Split View" }).click();
    await page.waitForTimeout(300);
    await expect(page.getByRole("button", { name: "Grid View" })).toBeVisible();
    await expect(page.getByText("Select an object from the tree")).toBeVisible();

    await page.getByText(heading).first().click();
    await page.waitForTimeout(500);
    await expect(page.locator("input[type='text']").first()).toBeVisible();
  });
});

test.describe("Batch Review", () => {
  test("review all marks unreviewed objects", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "br");
    await createObject(request, mod.id, `BR-A-${uid()}`);
    await createObject(request, mod.id, `BR-B-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    const reviewAllBtn = page.getByRole("button", { name: "Review All" });
    await expect(reviewAllBtn).toBeVisible();
    page.on("dialog", (dialog) => dialog.accept());
    await reviewAllBtn.click();
    await page.waitForTimeout(1000);
    const checkmarks = page.locator(".ag-row").locator("text=\u2705");
    await expect(checkmarks.first()).toBeVisible({ timeout: 5000 });
  });
});
