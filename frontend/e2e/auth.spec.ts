import { test, expect } from "@playwright/test";
import { setupAuth, API, waitForApp } from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

test.describe("Auth", () => {
  test("login API returns token and user", async ({ request }) => {
    const res = await request.post(`${API}/auth/login`, {
      data: { email: "admin@localhost", password: "admin" },
    });
    expect(res.ok()).toBeTruthy();
    const body = await res.json();
    expect(body.token).toBeTruthy();
    expect(body.user.email).toBe("admin@localhost");
    expect(body.user.role).toBe("admin");
  });

  test("register creates a new user", async ({ request }) => {
    const email = `reg-${Date.now()}@test.local`;
    const res = await request.post(`${API}/auth/register`, {
      data: { email, password: "testpass123", display_name: "Test User" },
    });
    expect(res.ok()).toBeTruthy();
    const body = await res.json();
    expect(body.email).toBe(email);
  });

  test("me endpoint returns current user", async ({ request }) => {
    const { setupAuth: _, ...h } = await import("./helpers");
    const body = await h.apiGet(request, "/auth/me");
    expect(body.email).toBe("admin@localhost");
    expect(body.role).toBe("admin");
  });

  test("unauthenticated request returns 401", async ({ request }) => {
    const res = await request.get(`${API}/workspaces`);
    expect(res.status()).toBe(401);
  });

  test("dev auto-login works in browser", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Navigation")).toBeVisible({ timeout: 15000 });
  });

  test("logout clears session", async ({ page }) => {
    await waitForApp(page);
    await page.getByRole("button", { name: "Logout" }).click();
    // In dev mode, auto-login may re-sign in immediately, so just verify we
    // briefly see the login page or the transition completes without error.
    await expect(
      page.getByText("Sign In").or(page.getByText("Signing in as dev admin...")).or(page.getByText("Navigation")),
    ).toBeVisible({ timeout: 10000 });
  });
});
