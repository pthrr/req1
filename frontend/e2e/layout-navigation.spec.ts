import { test, expect } from "@playwright/test";
import { setupAuth, waitForApp, createWorkspace, scaffold, getToken } from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

test.describe("Layout", () => {
  test("shows header with breadcrumbs and sidebar", async ({ page }) => {
    await waitForApp(page);
    await expect(page.locator("header")).toContainText("req1");
    await expect(page.getByText("Navigation")).toBeVisible();
    await expect(page.getByText("Welcome to req1")).toBeVisible();
  });

  test("sidebar can be collapsed and expanded", async ({ page }) => {
    await waitForApp(page);
    await expect(page.getByText("Navigation")).toBeVisible();
    await page.getByTitle("Collapse sidebar").click();
    await expect(page.getByText("Navigation")).not.toBeVisible();
    await page.getByTitle("Expand sidebar").click();
    await expect(page.getByText("Navigation")).toBeVisible();
  });

  test("direct URL navigation to workspace", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-url-${Date.now()}`);
    // Inject auth token before page loads to bypass login redirect
    await page.addInitScript((t) => { localStorage.setItem("token", t); }, getToken());
    await page.goto(`/w/${ws.id}`);
    await expect(page.getByText("Navigation")).toBeVisible({ timeout: 15000 });
    await expect(page.locator("main h1")).toContainText(ws.name, { timeout: 10000 });
  });

  test("direct URL navigation to module", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "url");
    // Inject auth token before page loads to bypass login redirect
    await page.addInitScript((t) => { localStorage.setItem("token", t); }, getToken());
    await page.goto(`/w/${ws.id}/p/${proj.id}/m/${mod.id}`);
    await expect(page.getByText("Navigation")).toBeVisible({ timeout: 15000 });
    await expect(page.getByRole("button", { name: "Objects" })).toBeVisible({ timeout: 10000 });
  });
});

test.describe("Breadcrumbs", () => {
  test("shows IDs for workspace/project/module", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "bc");
    const { navigateToModule } = await import("./helpers");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await expect(page.locator("header")).toContainText(ws.id.slice(0, 8));
    await expect(page.locator("header")).toContainText(proj.id.slice(0, 8));
    await expect(page.locator("header")).toContainText(mod.id.slice(0, 8));
  });
});

test.describe("Dark Mode", () => {
  test("toggle dark mode and persist preference", async ({ page }) => {
    await waitForApp(page);
    const toggle = page.getByRole("button", { name: /Dark|Light/ });
    await expect(toggle).toBeVisible();
    await expect(toggle).toHaveText("Dark");

    await toggle.click();
    await expect(toggle).toHaveText("Light");

    await page.reload();
    await expect(page.getByRole("button", { name: "Light" })).toBeVisible({ timeout: 15000 });

    await page.getByRole("button", { name: "Light" }).click();
    await expect(page.getByRole("button", { name: "Dark" })).toBeVisible();
  });
});
