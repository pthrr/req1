import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, waitForApp, navigateToModule, scaffold, clickRowAction,
  createObject, createLinkType, createLink,
  apiGet,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

// ---------------------------------------------------------------------------
// Global Search
// ---------------------------------------------------------------------------

test.describe("Global Search", () => {
  test("API: search objects globally", async ({ request }) => {
    const { mod } = await scaffold(request, "gsearch");
    const needle = `NEEDLE-${uid()}`;
    await createObject(request, mod.id, needle, `body with ${needle}`);
    const results = await apiGet(request, `/search?q=${needle}`);
    expect(results.items.length).toBeGreaterThan(0);
  });

  test("UI: global search in header", async ({ page, request }) => {
    const { mod } = await scaffold(request, "gsearchui");
    const needle = `SEARCH-${uid()}`;
    await createObject(request, mod.id, needle);
    await waitForApp(page);

    const searchInput = page.getByPlaceholder("Search all modules...");
    await searchInput.fill(needle);
    await page.waitForTimeout(500);
    await expect(page.getByText(needle).first()).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Module Full-Text Search
// ---------------------------------------------------------------------------

test.describe("Module Search", () => {
  test("UI: full-text search filters objects", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "ftsearch");
    const needle = `UNIQUE-${uid()}`;
    await createObject(request, mod.id, `OTHER-${uid()}`);
    await createObject(request, mod.id, needle, `body with ${needle} text`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByPlaceholder("Full-text search...").fill(needle);
    await page.getByRole("button", { name: "Search" }).click();
    await page.waitForTimeout(1000);

    const treePanel = page.locator("div").filter({ hasText: "Object Tree" }).first();
    await expect(treePanel).toBeVisible();

    await page.getByRole("button", { name: "Clear" }).click();
    await page.waitForTimeout(500);
  });
});

// ---------------------------------------------------------------------------
// Impact Analysis
// ---------------------------------------------------------------------------

test.describe("Impact Analysis", () => {
  test("API: get impact", async ({ request }) => {
    const { mod } = await scaffold(request, "impapi");
    const a = await createObject(request, mod.id, `IMP-A-${uid()}`);
    const b = await createObject(request, mod.id, `IMP-B-${uid()}`);
    const lt = await createLinkType(request, `imp-lt-${uid()}`);
    await createLink(request, a.id, b.id, lt.id);

    const impact = await apiGet(request, `/object-impact/${a.id}`);
    expect(impact).toBeTruthy();
  });

  test("UI: run impact analysis", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "impui");
    const obj1 = await createObject(request, mod.id, `IMP-A-${uid()}`);
    const obj2 = await createObject(request, mod.id, `IMP-B-${uid()}`);
    const lt = await createLinkType(request, `imp-lt-${uid()}`);
    await createLink(request, obj1.id, obj2.id, lt.id);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);
    await page.getByRole("button", { name: "Objects" }).click();
    await page.waitForTimeout(1000);
    await expect(page.locator(".ag-row").first()).toBeVisible({ timeout: 5000 });
    await clickRowAction(page, "Imp");
    await expect(page.getByText(/Impact Analysis for/)).toBeVisible({ timeout: 5000 });
    await page.getByRole("button", { name: /Analyze/ }).click();
    await expect(page.getByText(/Found.*linked object/)).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Coverage
// ---------------------------------------------------------------------------

test.describe("Coverage", () => {
  test("API: get coverage", async ({ request }) => {
    const { mod } = await scaffold(request, "covapi");
    await createObject(request, mod.id, `COV-${uid()}`);
    const cov = await apiGet(request, `/modules/${mod.id}/coverage`);
    expect(cov).toBeTruthy();
  });

  test("UI: shows coverage bars when objects have links", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "covui");
    const obj1 = await createObject(request, mod.id, `COV-A-${uid()}`);
    const obj2 = await createObject(request, mod.id, `COV-B-${uid()}`);
    const lt = await createLinkType(request, `cov-lt-${uid()}`);
    await createLink(request, obj1.id, obj2.id, lt.id);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);
    await expect(page.getByText("Link Coverage")).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Traceability Matrix
// ---------------------------------------------------------------------------

test.describe("Traceability Matrix", () => {
  test("API: get matrix", async ({ request }) => {
    const { mod } = await scaffold(request, "traceapi");
    await createObject(request, mod.id, `TM-${uid()}`);
    const matrix = await apiGet(
      request,
      `/traceability-matrix?source_module_id=${mod.id}&target_module_id=${mod.id}`,
    );
    expect(matrix).toBeTruthy();
  });

  test("UI: accessible from sidebar", async ({ page, request }) => {
    const { ws, proj } = await scaffold(request, "traceui");
    await waitForApp(page);
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await expect(page.getByText(proj.name)).toBeVisible({ timeout: 5000 });
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await expect(page.getByText("Traceability Matrix")).toBeVisible({ timeout: 5000 });
    await page.getByText("Traceability Matrix").click();
    await expect(page.locator("main h1")).toContainText("Traceability Matrix");
  });
});
