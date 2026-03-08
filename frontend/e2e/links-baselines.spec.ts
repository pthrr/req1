import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, navigateToModule, scaffold,
  createObject, createLinkType, createLink, createBaselineSet,
  apiGet, apiPost, apiPatch, apiDelete, auth, API,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

// ---------------------------------------------------------------------------
// Links
// ---------------------------------------------------------------------------

test.describe("Links", () => {
  test("API: create, list, delete link", async ({ request }) => {
    const { mod } = await scaffold(request, "linkcrud");
    const a = await createObject(request, mod.id, `L-A-${uid()}`);
    const b = await createObject(request, mod.id, `L-B-${uid()}`);
    const lt = await createLinkType(request, `lt-${uid()}`);
    const link = await createLink(request, a.id, b.id, lt.id);

    const list = await apiGet(request, "/links");
    expect(list.items.some((l: { id: string }) => l.id === link.id)).toBeTruthy();

    await apiDelete(request, `/links/${link.id}`);
  });

  test("API: update link (set suspect flag)", async ({ request }) => {
    const { mod } = await scaffold(request, "linkupd");
    const a = await createObject(request, mod.id, `LU-A-${uid()}`);
    const b = await createObject(request, mod.id, `LU-B-${uid()}`);
    const lt = await createLinkType(request, `lt-${uid()}`);
    const link = await createLink(request, a.id, b.id, lt.id);

    const updated = await apiPatch(request, `/links/${link.id}`, { suspect: true });
    expect(updated.suspect).toBe(true);

    const updated2 = await apiPatch(request, `/links/${link.id}`, { suspect: false });
    expect(updated2.suspect).toBe(false);

    await apiDelete(request, `/links/${link.id}`);
  });

  test("API: get single link", async ({ request }) => {
    const { mod } = await scaffold(request, "linkget");
    const a = await createObject(request, mod.id, `LG-A-${uid()}`);
    const b = await createObject(request, mod.id, `LG-B-${uid()}`);
    const lt = await createLinkType(request, `lt-${uid()}`);
    const link = await createLink(request, a.id, b.id, lt.id);

    const got = await apiGet(request, `/links/${link.id}`);
    expect(got.id).toBe(link.id);
    expect(got.source_object_id).toBe(a.id);
    expect(got.target_object_id).toBe(b.id);

    await apiDelete(request, `/links/${link.id}`);
  });

  test("API: list link types", async ({ request }) => {
    await createLinkType(request, `lt-list-${uid()}`);
    const body = await apiGet(request, "/link-types");
    expect(body.length).toBeGreaterThan(0);
  });

  test("UI: create and delete a link", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "linkui");
    const obj1 = await createObject(request, mod.id, `LNK-SRC-${uid()}`);
    const obj2 = await createObject(request, mod.id, `LNK-TGT-${uid()}`);
    const lt = await createLinkType(request, `lt-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Links" }).click();
    await page.waitForTimeout(500);

    const form = page.locator("form");
    const selects = form.locator("select");
    await selects.nth(0).selectOption(obj1.id);
    await selects.nth(1).selectOption(lt.id);
    await selects.nth(2).selectOption(obj2.id);
    await page.getByRole("button", { name: "Create Link" }).click();
    await page.waitForTimeout(500);
    await expect(page.locator("table tbody tr")).toHaveCount(1, { timeout: 3000 });

    await page.getByRole("button", { name: /Delete/ }).first().click();
    await page.waitForTimeout(500);
  });
});

// ---------------------------------------------------------------------------
// Baselines
// ---------------------------------------------------------------------------

test.describe("Baselines", () => {
  test("API: create, list, delete baseline", async ({ request }) => {
    const { mod } = await scaffold(request, "blcrud");
    await createObject(request, mod.id, `BL-${uid()}`);
    const bl = await apiPost(request, `/modules/${mod.id}/baselines`, { name: `BL-${uid()}` });
    expect(bl.id).toBeTruthy();

    const list = await apiGet(request, `/modules/${mod.id}/baselines`);
    expect(list.items.length).toBeGreaterThan(0);

    await apiDelete(request, `/modules/${mod.id}/baselines/${bl.id}`);
  });

  test("API: get single baseline with entries", async ({ request }) => {
    const { mod } = await scaffold(request, "blget");
    await createObject(request, mod.id, `BLG-${uid()}`);
    const bl = await apiPost(request, `/modules/${mod.id}/baselines`, { name: `BL-get-${uid()}` });

    const got = await apiGet(request, `/modules/${mod.id}/baselines/${bl.id}`);
    expect(got.id).toBe(bl.id);
    expect(got.entries).toBeTruthy();
    expect(got.entries.length).toBeGreaterThan(0);

    await apiDelete(request, `/modules/${mod.id}/baselines/${bl.id}`);
  });

  test("API: list all baselines (global)", async ({ request }) => {
    const { mod } = await scaffold(request, "blglobal");
    await createObject(request, mod.id, `BLG-${uid()}`);
    await apiPost(request, `/modules/${mod.id}/baselines`, { name: `BL-global-${uid()}` });

    const body = await apiGet(request, "/baselines");
    expect(body.items.length).toBeGreaterThan(0);
    expect(body.total).toBeGreaterThan(0);
  });

  test("API: global baseline-diff endpoint", async ({ request }) => {
    const { mod } = await scaffold(request, "bldiffg");
    await createObject(request, mod.id, `DIFFG-${uid()}`);
    const bl1 = await apiPost(request, `/modules/${mod.id}/baselines`, { name: `v1g-${uid()}` });
    await createObject(request, mod.id, `DIFFG2-${uid()}`);
    const bl2 = await apiPost(request, `/modules/${mod.id}/baselines`, { name: `v2g-${uid()}` });

    const res = await request.get(
      `${API}/baseline-diff?a=${bl1.id}&b=${bl2.id}`,
      { headers: auth() },
    );
    expect(res.ok()).toBeTruthy();
    const diff = await res.json();
    expect(diff.baseline_a).toBe(bl1.id);
    expect(diff.baseline_b).toBe(bl2.id);
    expect(diff.added).toBeTruthy();
  });

  test("API: baseline diff between two baselines", async ({ request }) => {
    const { mod } = await scaffold(request, "bldiffapi");
    await createObject(request, mod.id, `DIFF-${uid()}`);
    const bl1 = await apiPost(request, `/modules/${mod.id}/baselines`, { name: `v1-${uid()}` });
    await createObject(request, mod.id, `DIFF2-${uid()}`);
    const bl2 = await apiPost(request, `/modules/${mod.id}/baselines`, { name: `v2-${uid()}` });

    const res = await request.get(
      `${API}/modules/${mod.id}/baseline-diff?a=${bl1.id}&b=${bl2.id}`,
      { headers: auth() },
    );
    expect(res.ok()).toBeTruthy();
    const diff = await res.json();
    expect(diff).toBeTruthy();
  });

  test("UI: create two baselines and view diff", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "bldiff");
    await createObject(request, mod.id, `BL-OBJ-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Baselines" }).click();
    await page.waitForTimeout(300);

    await page.getByPlaceholder("Baseline name").fill("v1.0");
    await page.getByRole("button", { name: /Create/ }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText("v1.0")).toBeVisible({ timeout: 3000 });

    await page.getByRole("button", { name: "Objects" }).click();
    await page.waitForTimeout(300);
    await page.getByPlaceholder("Heading (e.g. REQ-001)").fill(`BL-NEW-${uid()}`);
    await page.getByRole("button", { name: "Add" }).click();
    await page.waitForTimeout(500);

    await page.getByRole("button", { name: "Baselines" }).click();
    await page.waitForTimeout(300);
    await page.getByPlaceholder("Baseline name").fill("v2.0");
    await page.getByRole("button", { name: /Create/ }).click();
    await page.waitForTimeout(500);
    await expect(page.getByRole("cell", { name: "v2.0" })).toBeVisible({ timeout: 3000 });

    const compareSelects = page.locator("select");
    await compareSelects.nth(0).selectOption({ label: "v1.0" });
    await compareSelects.nth(1).selectOption({ label: "v2.0" });
    await page.waitForTimeout(300);
    await page.getByRole("button", { name: /Diff/ }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(/added|modified|removed/i)).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Baseline Sets
// ---------------------------------------------------------------------------

test.describe("Baseline Sets", () => {
  test("API: list baseline sets", async ({ request }) => {
    await createBaselineSet(request, `Set-list-${uid()}`, "1.0");
    const body = await apiGet(request, "/baseline-sets");
    expect(body.items.length).toBeGreaterThan(0);
    expect(body.total).toBeGreaterThan(0);
  });

  test("API: create, read, update, delete", async ({ request }) => {
    const bs = await createBaselineSet(request, `Set-${uid()}`, "1.0");
    expect(bs.id).toBeTruthy();

    const got = await apiGet(request, `/baseline-sets/${bs.id}`);
    expect(got.name).toBe(bs.name);

    const updated = await apiPatch(request, `/baseline-sets/${bs.id}`, { version: "2.0" });
    expect(updated.version).toBe("2.0");

    await apiDelete(request, `/baseline-sets/${bs.id}`);
  });

  test("UI: create baseline set and filter baselines", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "bsui");
    await createObject(request, mod.id, `BSF-OBJ-${uid()}`);
    const bset = await createBaselineSet(request, `FilterSet-${uid()}`, "2.0");

    await apiPost(request, `/modules/${mod.id}/baselines`, {
      name: `BL-in-set-${uid()}`,
      baseline_set_id: bset.id,
    });
    await apiPost(request, `/modules/${mod.id}/baselines`, {
      name: `BL-no-set-${uid()}`,
    });

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Baselines" }).click();
    await page.waitForTimeout(500);

    const table = page.locator("table");
    await expect(table.locator("tbody tr")).toHaveCount(2, { timeout: 3000 });

    await page.getByRole("button", { name: new RegExp(`${bset.name}`) }).click();
    await page.waitForTimeout(300);
    await expect(table.locator("tbody tr")).toHaveCount(1, { timeout: 3000 });

    await page.getByRole("button", { name: "All" }).click();
    await page.waitForTimeout(300);
    await expect(table.locator("tbody tr")).toHaveCount(2, { timeout: 3000 });
  });
});
