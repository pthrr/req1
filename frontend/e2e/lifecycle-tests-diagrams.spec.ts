import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, navigateToModule, scaffold,
  apiGet, apiPost, apiPatch, apiDelete,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

// ---------------------------------------------------------------------------
// Lifecycle Models
// ---------------------------------------------------------------------------

test.describe("Lifecycle Models", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "lccrud");
    const lc = await apiPost(request, `/modules/${mod.id}/lifecycle-models`, {
      name: `LC-${uid()}`,
      initial_state: "draft",
      states: [
        { name: "draft", color: "#666666" },
        { name: "active", color: "#00aa00" },
      ],
      transitions: [{ from: "draft", to: "active" }],
    });
    expect(lc.id).toBeTruthy();

    const got = await apiGet(request, `/modules/${mod.id}/lifecycle-models/${lc.id}`);
    expect(got.name).toBeTruthy();

    const updated = await apiPatch(request, `/modules/${mod.id}/lifecycle-models/${lc.id}`, {
      name: `LC-upd-${uid()}`,
    });
    expect(updated.name).toContain("upd");

    await apiDelete(request, `/modules/${mod.id}/lifecycle-models/${lc.id}`);
  });

  test("UI: create lifecycle model", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "lcui");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Lifecycle" }).click();
    await page.waitForTimeout(300);

    await page.getByRole("button", { name: "New Model" }).click();
    await page.waitForTimeout(200);

    const modelName = `LC-${uid()}`;
    await page.getByPlaceholder("Model name").fill(modelName);
    await page.getByPlaceholder("Initial state").fill("draft");
    await page.getByRole("button", { name: "Create" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(modelName)).toBeVisible({ timeout: 3000 });
  });
});

// ---------------------------------------------------------------------------
// Test Cases
// ---------------------------------------------------------------------------

test.describe("Test Cases", () => {
  test("API: create, read, update, delete test case", async ({ request }) => {
    const { mod } = await scaffold(request, "tccrud");
    const tc = await apiPost(request, `/modules/${mod.id}/test-cases`, {
      name: `TC-${uid()}`,
      test_type: "manual",
      priority: "high",
    });
    expect(tc.id).toBeTruthy();

    const got = await apiGet(request, `/modules/${mod.id}/test-cases/${tc.id}`);
    expect(got.name).toBeTruthy();

    const updated = await apiPatch(request, `/modules/${mod.id}/test-cases/${tc.id}`, {
      name: `TC-upd-${uid()}`,
    });
    expect(updated.name).toContain("upd");

    await apiDelete(request, `/modules/${mod.id}/test-cases/${tc.id}`);
  });

  test("API: test execution CRUD", async ({ request }) => {
    const { mod } = await scaffold(request, "texec");
    const tc = await apiPost(request, `/modules/${mod.id}/test-cases`, {
      name: `TC-${uid()}`,
      test_type: "manual",
      priority: "medium",
    });

    const exec = await apiPost(request, `/test-cases/${tc.id}/executions`, {
      status: "passed",
      executor: "admin",
      environment: "dev",
    });
    expect(exec.id).toBeTruthy();

    const list = await apiGet(request, `/test-cases/${tc.id}/executions`);
    expect(list.length).toBe(1);

    // Get single execution
    const got = await apiGet(request, `/test-cases/${tc.id}/executions/${exec.id}`);
    expect(got.id).toBe(exec.id);
    expect(got.status).toBe("passed");

    // Update execution
    const updated = await apiPatch(request, `/test-cases/${tc.id}/executions/${exec.id}`, {
      status: "failed",
    });
    expect(updated.status).toBe("failed");

    // Delete execution
    await apiDelete(request, `/test-cases/${tc.id}/executions/${exec.id}`);
    const listAfter = await apiGet(request, `/test-cases/${tc.id}/executions`);
    expect(listAfter.length).toBe(0);
  });

  test("API: test coverage and dashboard", async ({ request }) => {
    const { mod } = await scaffold(request, "tcov");
    const coverage = await apiGet(request, `/modules/${mod.id}/test-coverage`);
    expect(coverage).toBeTruthy();

    const dashboard = await apiGet(request, `/modules/${mod.id}/test-dashboard`);
    expect(dashboard).toBeTruthy();
  });

  test("UI: create test case and record execution", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "tcui");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Tests" }).click();
    await page.waitForTimeout(300);

    const tcName = `TC-${uid()}`;
    await page.getByPlaceholder("Test case name").fill(tcName);
    await page.getByRole("button", { name: "Add Test Case" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(tcName)).toBeVisible({ timeout: 3000 });

    await page.getByText(tcName).click();
    await page.waitForTimeout(300);
    await page.getByPlaceholder("Executor").fill("admin");
    await page.getByRole("button", { name: "Record" }).click();
    await page.waitForTimeout(500);
  });
});

// ---------------------------------------------------------------------------
// Diagrams
// ---------------------------------------------------------------------------

test.describe("Diagrams", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "diagcrud");
    const diag = await apiPost(request, `/modules/${mod.id}/diagrams`, {
      name: `Diag-${uid()}`,
      diagram_type: "use_case",
      source_code: "graph TD\n  A-->B",
    });
    expect(diag.id).toBeTruthy();

    const got = await apiGet(request, `/modules/${mod.id}/diagrams/${diag.id}`);
    expect(got.name).toBeTruthy();

    const updated = await apiPatch(request, `/modules/${mod.id}/diagrams/${diag.id}`, {
      name: `Diag-upd-${uid()}`,
    });
    expect(updated.name).toContain("upd");

    await apiDelete(request, `/modules/${mod.id}/diagrams/${diag.id}`);
  });

  test("UI: create diagram", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "diagui");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Diagrams" }).click();
    await page.waitForTimeout(300);

    const diagName = `Diag-${uid()}`;
    await page.getByPlaceholder("Diagram name").fill(diagName);
    await page.getByRole("button", { name: "Create Diagram" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(diagName)).toBeVisible({ timeout: 3000 });
  });
});
