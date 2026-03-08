import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, waitForApp, scaffold,
  createObject, createWorkspace,
  apiGet, apiPost, apiPatch, apiDelete, auth, API,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

// ---------------------------------------------------------------------------
// Webhooks
// ---------------------------------------------------------------------------

test.describe("Webhooks", () => {
  test("API: create, list, update, delete", async ({ request }) => {
    const { mod } = await scaffold(request, "whcrud");
    const wh = await apiPost(request, `/modules/${mod.id}/webhooks`, {
      name: `wh-${uid()}`,
      url: "https://example.com/hook",
      events: "object.created",
    });
    expect(wh.id).toBeTruthy();

    const list = await apiGet(request, `/modules/${mod.id}/webhooks`);
    expect(list.length).toBe(1);

    // Get by ID
    const got = await apiGet(request, `/modules/${mod.id}/webhooks/${wh.id}`);
    expect(got.id).toBe(wh.id);
    expect(got.url).toBe("https://example.com/hook");

    const updated = await apiPatch(request, `/modules/${mod.id}/webhooks/${wh.id}`, {
      url: "https://example.com/hook2",
    });
    expect(updated.url).toContain("hook2");

    await apiDelete(request, `/modules/${mod.id}/webhooks/${wh.id}`);
  });
});

// ---------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------

test.describe("Users", () => {
  test("API: create, list, update, delete", async ({ request }) => {
    const email = `user-${uid()}@test.local`;
    const user = await apiPost(request, "/users", {
      email,
      display_name: "Test User",
    });
    expect(user.id).toBeTruthy();

    const list = await apiGet(request, "/users");
    expect(list.items.some((u: { id: string }) => u.id === user.id)).toBeTruthy();

    // Get by ID
    const got = await apiGet(request, `/users/${user.id}`);
    expect(got.id).toBe(user.id);
    expect(got.email).toBe(email);

    const updated = await apiPatch(request, `/users/${user.id}`, {
      display_name: "Updated User",
    });
    expect(updated.display_name).toBe("Updated User");

    await apiDelete(request, `/users/${user.id}`);
  });
});

// ---------------------------------------------------------------------------
// Audit Log
// ---------------------------------------------------------------------------

test.describe("Audit Log", () => {
  test("API: list audit entries", async ({ request }) => {
    const body = await apiGet(request, "/audit-log");
    expect(body.items).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

test.describe("Notifications", () => {
  test("API: list notifications", async ({ request }) => {
    const body = await apiGet(request, "/notifications");
    expect(body.items).toBeTruthy();
  });

  test("API: unread count", async ({ request }) => {
    const body = await apiGet(request, "/notifications/unread-count");
    expect(body.count).toBeDefined();
  });

  test("API: mark all notifications read", async ({ request }) => {
    const res = await request.post(`${API}/notifications/read-all`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();

    const count = await apiGet(request, "/notifications/unread-count");
    expect(count.count).toBe(0);
  });

  test("API: mark single notification read", async ({ request }) => {
    // List notifications and mark the first one read (if any exist)
    const body = await apiGet(request, "/notifications");
    if (body.items.length > 0) {
      const notif = body.items[0];
      const res = await request.post(`${API}/notifications/${notif.id}/read`, {
        headers: auth(),
      });
      expect(res.ok()).toBeTruthy();
      const updated = await res.json();
      expect(updated.id).toBe(notif.id);
    }
  });

  test("UI: notification bell is visible", async ({ page }) => {
    await waitForApp(page);
    await expect(page.getByTitle("Notifications")).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// E-Signatures
// ---------------------------------------------------------------------------

test.describe("E-Signatures", () => {
  test("API: create and list signatures", async ({ request }) => {
    const { mod } = await scaffold(request, "esig");
    const obj = await createObject(request, mod.id, `SIG-${uid()}`);

    const sig = await apiPost(request, "/e-signatures", {
      entity_type: "object",
      entity_id: obj.id,
      meaning: "approved",
      password: "admin",
    });
    expect(sig.id).toBeTruthy();

    const list = await apiGet(request, `/e-signatures/entity/object/${obj.id}`);
    expect(list.length).toBeGreaterThan(0);
  });
});

// ---------------------------------------------------------------------------
// Dashboards & Widgets
// ---------------------------------------------------------------------------

test.describe("Dashboards", () => {
  test("API: create, list, update, delete dashboard", async ({ request }) => {
    const ws = await createWorkspace(request, `WS-dash-${uid()}`);
    const dash = await apiPost(request, `/workspaces/${ws.id}/dashboards`, {
      name: `Dash-${uid()}`,
    });
    expect(dash.id).toBeTruthy();

    const list = await apiGet(request, `/workspaces/${ws.id}/dashboards`);
    expect(list.length).toBe(1);

    const updated = await apiPatch(request, `/workspaces/${ws.id}/dashboards/${dash.id}`, {
      name: `Dash-upd-${uid()}`,
    });
    expect(updated.name).toContain("upd");

    await apiDelete(request, `/workspaces/${ws.id}/dashboards/${dash.id}`);
  });

  test("API: widget CRUD", async ({ request }) => {
    const ws = await createWorkspace(request, `WS-wid-${uid()}`);
    const dash = await apiPost(request, `/workspaces/${ws.id}/dashboards`, {
      name: `Dash-${uid()}`,
    });

    // Get dashboard by ID
    const gotDash = await apiGet(request, `/workspaces/${ws.id}/dashboards/${dash.id}`);
    expect(gotDash.id).toBe(dash.id);

    const widget = await apiPost(request, `/dashboards/${dash.id}/widgets`, {
      title: `Widget-${uid()}`,
      widget_type: "coverage_chart",
    });
    expect(widget.id).toBeTruthy();

    const list = await apiGet(request, `/dashboards/${dash.id}/widgets`);
    expect(list.length).toBe(1);

    // Get widget by ID
    const gotWidget = await apiGet(request, `/dashboards/${dash.id}/widgets/${widget.id}`);
    expect(gotWidget.id).toBe(widget.id);

    await apiDelete(request, `/dashboards/${dash.id}/widgets/${widget.id}`);
  });

  test("UI: create dashboard and add widget", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-dashui-${uid()}`);
    await waitForApp(page);
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText("Dashboards").click();
    await page.waitForTimeout(500);

    const dashName = `Dash-${uid()}`;
    await page.getByPlaceholder("Dashboard name").fill(dashName);
    await page.getByRole("button", { name: "Create" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(dashName)).toBeVisible({ timeout: 3000 });

    await page.getByText(dashName).click();
    await page.waitForTimeout(500);
    const widgetTitle = `Widget-${uid()}`;
    await page.getByPlaceholder("Widget title").fill(widgetTitle);
    await page.getByRole("button", { name: "Add Widget" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(widgetTitle)).toBeVisible({ timeout: 3000 });
  });
});

// ---------------------------------------------------------------------------
// Project Templates
// ---------------------------------------------------------------------------

test.describe("Project Templates", () => {
  test("API: create, list, delete template", async ({ request }) => {
    const tpl = await apiPost(request, "/project-templates", {
      name: `Tpl-${uid()}`,
      standard: "custom",
      template_data: { modules: [{ name: "Module A" }] },
    });
    expect(tpl.id).toBeTruthy();

    const list = await apiGet(request, "/project-templates");
    expect(list.some((t: { id: string }) => t.id === tpl.id)).toBeTruthy();

    await apiDelete(request, `/project-templates/${tpl.id}`);
  });

  test("API: update template", async ({ request }) => {
    const tpl = await apiPost(request, "/project-templates", {
      name: `Tpl-${uid()}`,
      standard: "custom",
      template_data: { modules: [{ name: "Module A" }] },
    });

    const updated = await apiPatch(request, `/project-templates/${tpl.id}`, {
      name: `Tpl-upd-${uid()}`,
    });
    expect(updated.name).toContain("upd");

    // Get by ID
    const got = await apiGet(request, `/project-templates/${tpl.id}`);
    expect(got.id).toBe(tpl.id);

    await apiDelete(request, `/project-templates/${tpl.id}`);
  });

  test("API: instantiate template", async ({ request }) => {
    const ws = await createWorkspace(request, `WS-inst-${uid()}`);
    const tpl = await apiPost(request, "/project-templates", {
      name: `Tpl-inst-${uid()}`,
      standard: "custom",
      template_data: { modules: [{ name: "Module A" }, { name: "Module B" }] },
    });

    const res = await request.post(`${API}/project-templates/${tpl.id}/instantiate`, {
      data: {
        workspace_id: ws.id,
        project_name: `Proj-inst-${uid()}`,
      },
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    const result = await res.json();
    expect(result.project_id).toBeTruthy();
    expect(result.modules_created).toBe(2);

    await apiDelete(request, `/project-templates/${tpl.id}`);
  });

  test("UI: template wizard visible from sidebar", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-tplwiz-${uid()}`);
    await waitForApp(page);
    await page.getByText(ws.name).hover();
    await page.waitForTimeout(200);
    await expect(page.getByTitle("From Template")).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// ReqIF Export
// ---------------------------------------------------------------------------

test.describe("ReqIF", () => {
  test("API: export module as ReqIF", async ({ request }) => {
    const { mod } = await scaffold(request, "reqifexp");
    await createObject(request, mod.id, `REQIF-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/reqif/export`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
  });

  test("API: export module as ReqIF archive (.reqifz)", async ({ request }) => {
    const { mod } = await scaffold(request, "reqifz");
    await createObject(request, mod.id, `REQIFZ-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/reqif/export?format=reqifz`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    const contentType = res.headers()["content-type"];
    expect(contentType).toContain("zip");
  });
});

// ---------------------------------------------------------------------------
// Health Checks
// ---------------------------------------------------------------------------

test.describe("Health", () => {
  test("API: liveness probe", async ({ request }) => {
    const res = await request.get("http://localhost:8080/health/live");
    expect(res.ok()).toBeTruthy();
  });

  test("API: readiness probe", async ({ request }) => {
    const res = await request.get("http://localhost:8080/health/ready");
    expect(res.ok()).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Auth: change password
// ---------------------------------------------------------------------------

test.describe("Auth", () => {
  test("API: change password (and change back)", async ({ request }) => {
    const res = await request.post(`${API}/auth/change-password`, {
      data: { old_password: "admin", new_password: "admin2" },
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();

    // Change it back so other tests keep working
    const res2 = await request.post(`${API}/auth/change-password`, {
      data: { old_password: "admin2", new_password: "admin" },
      headers: auth(),
    });
    expect(res2.ok()).toBeTruthy();
  });
});
