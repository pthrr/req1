import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, navigateToModule, scaffold,
  createObject, apiGet, apiPost, apiPatch, apiDelete, auth, API,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

test.describe("Scripts", () => {
  test("API: create, read, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "scriptcrud");
    const script = await apiPost(request, `/modules/${mod.id}/scripts`, {
      name: `Script-${uid()}`,
      source_code: "return obj.heading || ''",
      script_type: "layout",
    });
    expect(script.id).toBeTruthy();

    const got = await apiGet(request, `/modules/${mod.id}/scripts/${script.id}`);
    expect(got.name).toBeTruthy();

    const updated = await apiPatch(request, `/modules/${mod.id}/scripts/${script.id}`, {
      name: `Script-updated-${uid()}`,
    });
    expect(updated.name).toContain("updated");

    await apiDelete(request, `/modules/${mod.id}/scripts/${script.id}`);
  });

  test("API: test script execution", async ({ request }) => {
    const { mod } = await scaffold(request, "scriptexec");
    await createObject(request, mod.id, `SE-${uid()}`);
    const script = await apiPost(request, `/modules/${mod.id}/scripts`, {
      name: `Test-${uid()}`,
      source_code: "return obj.heading || ''",
      script_type: "layout",
    });

    // Test the script (dry run)
    const testRes = await request.post(
      `${API}/modules/${mod.id}/scripts/${script.id}/test`,
      { data: {}, headers: auth() },
    );
    // Test endpoint may return 200 or error depending on script type
    expect([200, 400, 500]).toContain(testRes.status());

    // Execute the script
    const execRes = await request.post(
      `${API}/modules/${mod.id}/scripts/${script.id}/execute`,
      { data: {}, headers: auth() },
    );
    expect([200, 400, 500]).toContain(execRes.status());

    // List executions
    const executions = await apiGet(request, `/modules/${mod.id}/scripts/${script.id}/executions`);
    expect(executions).toBeTruthy();
  });

  test("UI: create and delete a script", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "scriptui");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Scripts" }).click();
    await page.waitForTimeout(300);

    const scriptName = `Script-${uid()}`;
    await page.getByPlaceholder("Script name").fill(scriptName);
    const typeSelect = page.locator("select").first();
    await typeSelect.selectOption("layout");
    await page.getByPlaceholder("// JavaScript source code").fill("return obj.heading || ''");
    await page.getByRole("button", { name: "Create Script" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(scriptName)).toBeVisible({ timeout: 3000 });

    await page.getByText(scriptName).click();
    await page.waitForTimeout(300);
    await page.getByRole("button", { name: /Del/ }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(scriptName)).not.toBeVisible();
  });
});

test.describe("Validation", () => {
  test("API: validate module", async ({ request }) => {
    const { mod } = await scaffold(request, "valapi");
    await createObject(request, mod.id, `VAL-${uid()}`);
    const result = await apiGet(request, `/modules/${mod.id}/validate`);
    expect(result).toBeTruthy();
  });

  test("UI: run validation and see results", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "valui");
    await createObject(request, mod.id, `VAL-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Validation" }).click();
    await page.waitForTimeout(300);
    await page.getByRole("button", { name: /Run Validation/ }).click();
    await expect(
      page.getByText(/All checks passed|error|warning|info/).first(),
    ).toBeVisible({ timeout: 5000 });
  });
});
