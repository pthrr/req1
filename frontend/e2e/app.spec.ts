import { test, expect } from "@playwright/test";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const API = "http://localhost:8080/api/v1";

/** Create a workspace via API, return its id + name. */
async function createWorkspace(request: import("@playwright/test").APIRequestContext, name: string) {
  const res = await request.post(`${API}/workspaces`, { data: { name } });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string; name: string };
}

/** Create a project via API. */
async function createProject(request: import("@playwright/test").APIRequestContext, wsId: string, name: string) {
  const res = await request.post(`${API}/workspaces/${wsId}/projects`, { data: { name } });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string; name: string };
}

/** Create a module via API. */
async function createModule(request: import("@playwright/test").APIRequestContext, projectId: string, name: string) {
  const res = await request.post(`${API}/modules`, { data: { name, project_id: projectId } });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string; name: string };
}

/** Create an object via API. */
async function createObject(
  request: import("@playwright/test").APIRequestContext,
  moduleId: string,
  heading: string,
  body?: string,
) {
  const res = await request.post(`${API}/modules/${moduleId}/objects`, {
    data: { heading, body },
  });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string };
}

// Generate a unique suffix to avoid test collisions
function uid() {
  return Math.random().toString(36).slice(2, 8);
}

// ---------------------------------------------------------------------------
// Layout & Sidebar
// ---------------------------------------------------------------------------

test.describe("Layout", () => {
  test("shows header with breadcrumbs and sidebar", async ({ page }) => {
    await page.goto("/");

    // Header exists with "req1" text
    await expect(page.locator("header")).toContainText("req1");

    // Sidebar "Navigation" label visible
    await expect(page.getByText("Navigation")).toBeVisible();

    // Welcome screen
    await expect(page.getByText("Welcome to req1")).toBeVisible();
  });

  test("sidebar can be collapsed and expanded", async ({ page }) => {
    await page.goto("/");

    // Sidebar is visible
    await expect(page.getByText("Navigation")).toBeVisible();

    // Collapse
    await page.getByTitle("Collapse sidebar").click();
    await expect(page.getByText("Navigation")).not.toBeVisible();

    // Expand
    await page.getByTitle("Expand sidebar").click();
    await expect(page.getByText("Navigation")).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Workspace CRUD via Sidebar
// ---------------------------------------------------------------------------

test.describe("Workspace CRUD", () => {
  test("create workspace from sidebar", async ({ page }) => {
    await page.goto("/");
    const name = `WS-${uid()}`;

    // Click "+" next to Navigation
    await page.locator("div").filter({ hasText: /^Navigation/ }).getByTitle("New workspace").click();

    // Type name and enter
    await page.getByPlaceholder("Workspace name").fill(name);
    await page.getByPlaceholder("Workspace name").press("Enter");

    // Workspace should appear in sidebar
    await expect(page.getByText(name)).toBeVisible();
  });

  test("select workspace shows detail", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-detail-${uid()}`);
    await page.goto("/");
    await page.waitForTimeout(500); // wait for sidebar to load

    // Click on workspace
    await page.getByText(ws.name).click();

    // Breadcrumbs show truncated workspace ID
    await expect(page.locator("header")).toContainText(ws.id.slice(0, 8));

    // Main content shows workspace name
    await expect(page.locator("main h1")).toContainText(ws.name);
  });

  test("rename workspace from sidebar", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-rename-${uid()}`);
    const newName = `WS-renamed-${uid()}`;
    await page.goto("/");
    await page.waitForTimeout(500);

    // Hover to reveal edit button
    await page.getByText(ws.name, { exact: true }).hover();
    await page.waitForTimeout(200);

    // Click the pencil (rename) button
    await page.getByTitle("Rename").first().click();
    await page.waitForTimeout(200);

    // After clicking rename, the text is replaced by an input pre-filled with the name.
    // Find the focused input containing the old name.
    const input = page.locator("input").filter({ hasValue: ws.name });
    await expect(input).toBeVisible({ timeout: 3000 });
    await input.fill(newName);
    await input.press("Enter");

    await expect(page.getByText(newName)).toBeVisible({ timeout: 3000 });
  });

  test("delete workspace from sidebar", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-delete-${uid()}`);
    await page.goto("/");
    await page.waitForTimeout(500);

    // Hover to reveal actions
    await page.getByText(ws.name).hover();
    await page.getByTitle("Delete").first().click();

    // Should disappear
    await expect(page.getByText(ws.name)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Project CRUD via Sidebar
// ---------------------------------------------------------------------------

test.describe("Project CRUD", () => {
  test("create project under workspace", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-proj-${uid()}`);
    const projName = `Proj-${uid()}`;
    await page.goto("/");
    await page.waitForTimeout(500);

    // Expand workspace
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);

    // Hover workspace to reveal "+" for project
    await page.getByText(ws.name).hover();
    await page.getByTitle("Add project").click();

    // Fill name
    await page.getByPlaceholder("Project name").fill(projName);
    await page.getByPlaceholder("Project name").press("Enter");

    await expect(page.getByText(projName)).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Module navigation & tabs
// ---------------------------------------------------------------------------

test.describe("Module navigation", () => {
  test("selecting a module shows ModuleDetail with tabs", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-mod-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-mod-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-${uid()}`);
    await page.goto("/");
    await page.waitForTimeout(500);

    // Expand workspace
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);

    // Expand project
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);

    // Click module
    await page.getByText(mod.name).click();

    // Tab bar should be visible
    await expect(page.getByRole("button", { name: "Objects" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Links" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Baselines" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Attributes" })).toBeVisible();

    // Breadcrumbs should show truncated IDs
    await expect(page.locator("header")).toContainText(ws.id.slice(0, 8));
    await expect(page.locator("header")).toContainText(proj.id.slice(0, 8));
    await expect(page.locator("header")).toContainText(mod.id.slice(0, 8));
  });
});

// ---------------------------------------------------------------------------
// Tabs switching
// ---------------------------------------------------------------------------

test.describe("Tabs", () => {
  let wsName: string;
  let projName: string;
  let modName: string;

  test.beforeEach(async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-tab-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-tab-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-tab-${uid()}`);
    wsName = ws.name;
    projName = proj.name;
    modName = mod.name;

    await page.goto("/");
    await page.waitForTimeout(500);

    // Navigate to module
    await page.getByText(wsName).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(projName).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(modName).click();
    await page.waitForTimeout(300);
  });

  test("Objects tab is default with search bar", async ({ page }) => {
    await expect(page.getByPlaceholder("Full-text search...")).toBeVisible();
  });

  test("switching to Links tab shows link form", async ({ page }) => {
    await page.getByRole("button", { name: "Links" }).click();
    await expect(page.getByRole("button", { name: "Create Link" })).toBeVisible();
  });

  test("switching to Baselines tab shows baseline form", async ({ page }) => {
    await page.getByRole("button", { name: "Baselines" }).click();
    await expect(page.getByPlaceholder("Baseline name")).toBeVisible();
  });

  test("switching to Attributes tab shows attr form", async ({ page }) => {
    await page.getByRole("button", { name: "Attributes" }).click();
    await expect(page.getByPlaceholder("Attribute name")).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Object creation in ag-grid
// ---------------------------------------------------------------------------

test.describe("Objects", () => {
  test("create object and see it in grid", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-obj-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-obj-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-obj-${uid()}`);

    await page.goto("/");
    await page.waitForTimeout(500);

    // Navigate to module
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(mod.name).click();
    await page.waitForTimeout(500);

    const heading = `REQ-${uid()}`;
    await page.getByPlaceholder("Heading (e.g. REQ-001)").fill(heading);
    await page.getByPlaceholder("Body", { exact: true }).fill("Test body text");
    await page.getByRole("button", { name: "Add" }).click();

    // Object should appear in grid
    await expect(page.locator(".ag-root-wrapper")).toContainText(heading, { timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Object History Modal
// ---------------------------------------------------------------------------

test.describe("Object History Modal", () => {
  test("clicking a row opens history modal", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-hist-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-hist-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-hist-${uid()}`);
    await createObject(request, mod.id, `REQ-HIST-${uid()}`, "History test body");

    await page.goto("/");
    await page.waitForTimeout(500);

    // Navigate to module
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(mod.name).click();
    await page.waitForTimeout(1000);

    // Click on a row in the grid
    await page.locator(".ag-row").first().click();

    // Modal should appear with "History for" text
    await expect(page.getByText("History for")).toBeVisible({ timeout: 5000 });

    // Close button should work
    await page.getByRole("button", { name: "Close" }).click();
    await expect(page.getByText("History for")).not.toBeVisible();
  });

  test("clicking overlay backdrop closes modal", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-histbg-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-histbg-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-histbg-${uid()}`);
    await createObject(request, mod.id, `REQ-HISTBG-${uid()}`);

    await page.goto("/");
    await page.waitForTimeout(500);

    // Navigate
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);
    await page.getByText(mod.name).click();
    await page.waitForTimeout(1000);

    // Click row
    await page.locator(".ag-row").first().click();
    await expect(page.getByText("History for")).toBeVisible({ timeout: 5000 });

    // Click backdrop (the overlay div at position 0,0)
    await page.locator("div[style*='position: fixed']").click({ position: { x: 5, y: 5 } });
    await expect(page.getByText("History for")).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Traceability Matrix
// ---------------------------------------------------------------------------

test.describe("Traceability Matrix", () => {
  test("accessible from sidebar under project", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-trace-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-trace-${uid()}`);
    await createModule(request, proj.id, `Mod-trace-${uid()}`);

    // Load page after data is created so sidebar fetches it
    await page.goto("/");
    await expect(page.getByText(ws.name)).toBeVisible({ timeout: 5000 });

    // Expand workspace by clicking the chevron
    const wsChevron = page.getByText(ws.name).locator("..").locator("span").first();
    await wsChevron.click();
    await expect(page.getByText(proj.name)).toBeVisible({ timeout: 5000 });

    // Expand project
    const projChevron = page.getByText(proj.name).locator("..").locator("span").first();
    await projChevron.click();
    await expect(page.getByText("Traceability Matrix")).toBeVisible({ timeout: 5000 });

    // Click Traceability Matrix
    await page.getByText("Traceability Matrix").click();

    // Should show the matrix page
    await expect(page.locator("main h1")).toContainText("Traceability Matrix");

    // Breadcrumbs should include "Traceability Matrix"
    await expect(page.locator("header")).toContainText("Traceability Matrix");
  });
});

// ---------------------------------------------------------------------------
// Helpers (additional)
// ---------------------------------------------------------------------------

/** Create a link type via API. */
async function createLinkType(request: import("@playwright/test").APIRequestContext, name: string) {
  const res = await request.post(`${API}/link-types`, { data: { name } });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string; name: string };
}

/** Create a link via API. */
async function createLink(
  request: import("@playwright/test").APIRequestContext,
  sourceId: string,
  targetId: string,
  linkTypeId: string,
) {
  const res = await request.post(`${API}/links`, {
    data: { source_object_id: sourceId, target_object_id: targetId, link_type_id: linkTypeId },
  });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string };
}

/** Create an attribute definition via API. */
async function createAttrDef(
  request: import("@playwright/test").APIRequestContext,
  moduleId: string,
  name: string,
  dataType: string,
) {
  const res = await request.post(`${API}/modules/${moduleId}/attribute-definitions`, {
    data: { name, data_type: dataType },
  });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string };
}

/** Navigate from / to a module's detail page. */
async function navigateToModule(
  page: import("@playwright/test").Page,
  wsName: string,
  projName: string,
  modName: string,
) {
  await page.goto("/");
  await page.waitForTimeout(500);
  await page.getByText(wsName).locator("..").locator("span").first().click();
  await page.waitForTimeout(300);
  await page.getByText(projName).locator("..").locator("span").first().click();
  await page.waitForTimeout(300);
  await page.getByText(modName).click();
  await page.waitForTimeout(500);
}

// ---------------------------------------------------------------------------
// Scripts Tab
// ---------------------------------------------------------------------------

test.describe("Scripts Tab", () => {
  test("create and delete a script", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-script-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-script-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-script-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Scripts" }).click();
    await page.waitForTimeout(300);

    // Create a layout script
    const scriptName = `Script-${uid()}`;
    await page.getByPlaceholder("Script name").fill(scriptName);
    // Select layout type
    const typeSelect = page.locator("select").first();
    await typeSelect.selectOption("layout");
    await page.getByPlaceholder("-- Lua source code").fill("return obj.heading or ''");
    await page.getByRole("button", { name: "Create Script" }).click();
    await page.waitForTimeout(500);

    // Script should appear in the list
    await expect(page.getByText(scriptName)).toBeVisible({ timeout: 3000 });

    // Select it and delete
    await page.getByText(scriptName).click();
    await page.waitForTimeout(300);
    // Find and click Del button
    await page.getByRole("button", { name: /Del/ }).click();
    await page.waitForTimeout(500);

    // Should be removed
    await expect(page.getByText(scriptName)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Validation Tab
// ---------------------------------------------------------------------------

test.describe("Validation Tab", () => {
  test("run validation and see results", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-val-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-val-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-val-${uid()}`);
    await createObject(request, mod.id, `VAL-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Validation" }).click();
    await page.waitForTimeout(300);

    // Click run button
    await page.getByRole("button", { name: /Run Validation/ }).click();

    // Wait for results — use first() since multiple elements match
    await expect(
      page.getByText(/All checks passed|error|warning|info/).first(),
    ).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Types Tab
// ---------------------------------------------------------------------------

test.describe("Types Tab", () => {
  test("create and delete an object type", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-type-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-type-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-type-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Types" }).click();
    await page.waitForTimeout(300);

    // Create object type
    const typeName = `Type-${uid()}`;
    await page.getByPlaceholder("Type name").fill(typeName);
    await page.getByRole("button", { name: /^Add$/ }).click();
    await page.waitForTimeout(500);

    // Should appear in table
    await expect(page.getByText(typeName)).toBeVisible({ timeout: 3000 });

    // Delete it
    await page.getByRole("button", { name: /Delete/ }).first().click();
    await page.waitForTimeout(500);

    // Should be removed
    await expect(page.getByText(typeName)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Settings Tab
// ---------------------------------------------------------------------------

test.describe("Settings Tab", () => {
  test("settings form is visible and saveable", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-set-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-set-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-set-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Settings" }).click();
    await page.waitForTimeout(300);

    // Verify settings form elements
    await expect(page.getByPlaceholder("e.g. REQ")).toBeVisible();
    await expect(page.getByPlaceholder("e.g. -")).toBeVisible();
    await expect(page.getByRole("button", { name: /Save Settings/ })).toBeVisible();

    // Change prefix and save
    await page.getByPlaceholder("e.g. REQ").fill("SPEC");
    await page.getByRole("button", { name: /Save Settings/ }).click();

    // "Saved" indicator should appear (transient — shown for 2s)
    await expect(page.getByText("Saved")).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Comments Panel
// ---------------------------------------------------------------------------

test.describe("Comments Panel", () => {
  test("add and resolve a comment", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-cmt-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-cmt-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-cmt-${uid()}`);
    await createObject(request, mod.id, `REQ-CMT-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click on the row to open history, then close it, then click Cmt button
    // The ag-grid row click opens history; we need to close it first
    const cmtBtn = page.getByRole("button", { name: "Cmt" }).first();
    await cmtBtn.click();
    await page.waitForTimeout(500);

    // If history opened instead, close it and retry
    if (await page.getByText(/History for/).isVisible()) {
      await page.getByRole("button", { name: "Close" }).click();
      await page.waitForTimeout(300);
      await cmtBtn.click();
      await page.waitForTimeout(500);
    }

    // Panel should be visible
    await expect(page.getByText(/Comments for/)).toBeVisible({ timeout: 5000 });

    // Add a comment
    await page.getByPlaceholder("Write a comment...").fill("Please review this item");
    await page.getByRole("button", { name: /Add Comment/ }).click();
    await page.waitForTimeout(500);

    // Comment should appear
    await expect(page.getByText("Please review this item")).toBeVisible();

    // Resolve
    await page.getByRole("button", { name: /Resolve/ }).first().click();
    await page.waitForTimeout(300);

    // "Resolved" should appear (button text changes)
    await expect(page.getByRole("button", { name: /Resolved/ })).toBeVisible();

    // Close panel
    await page.getByRole("button", { name: "Close" }).click();
  });
});

// ---------------------------------------------------------------------------
// Impact Panel
// ---------------------------------------------------------------------------

test.describe("Impact Panel", () => {
  test("run impact analysis", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-imp-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-imp-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-imp-${uid()}`);
    const obj1 = await createObject(request, mod.id, `IMP-A-${uid()}`);
    const obj2 = await createObject(request, mod.id, `IMP-B-${uid()}`);
    const lt = await createLinkType(request, `imp-lt-${uid()}`);
    await createLink(request, obj1.id, obj2.id, lt.id);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click the "Imp" button (ag-grid row click may open history first)
    const impBtn = page.getByRole("button", { name: "Imp" }).first();
    await impBtn.click();
    await page.waitForTimeout(500);

    // If history opened instead, close it and retry
    if (await page.getByText(/History for/).isVisible()) {
      await page.getByRole("button", { name: "Close" }).click();
      await page.waitForTimeout(300);
      await impBtn.click();
      await page.waitForTimeout(500);
    }

    // Panel should show
    await expect(page.getByText(/Impact Analysis for/)).toBeVisible({ timeout: 5000 });

    // Click Analyze
    await page.getByRole("button", { name: /Analyze/ }).click();

    // Wait for results
    await expect(page.getByText(/Found.*linked object/)).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Coverage Widget
// ---------------------------------------------------------------------------

test.describe("Coverage Widget", () => {
  test("shows coverage bars when objects have links", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-cov-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-cov-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-cov-${uid()}`);
    const obj1 = await createObject(request, mod.id, `COV-A-${uid()}`);
    const obj2 = await createObject(request, mod.id, `COV-B-${uid()}`);
    const lt = await createLinkType(request, `cov-lt-${uid()}`);
    await createLink(request, obj1.id, obj2.id, lt.id);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Coverage widget should be visible in Objects tab
    await expect(page.getByText("Link Coverage")).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Module Templates (Sidebar)
// ---------------------------------------------------------------------------

test.describe("Module Templates", () => {
  test("create module from template", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-tpl-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-tpl-${uid()}`);
    const templateMod = await createModule(request, proj.id, `Template-${uid()}`);
    // Add an attribute definition to the template module
    await createAttrDef(request, templateMod.id, "priority", "string");

    await page.goto("/");
    await page.waitForTimeout(500);

    // Expand workspace
    await page.getByText(ws.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);

    // Expand project
    await page.getByText(proj.name).locator("..").locator("span").first().click();
    await page.waitForTimeout(300);

    // Hover project to reveal Add module button
    await page.getByText(proj.name).hover();
    await page.getByTitle("Add module").click();
    await page.waitForTimeout(300);

    // Fill module name
    const newModName = `FromTpl-${uid()}`;
    await page.getByPlaceholder("Module name").fill(newModName);

    // Select template
    const templateSelect = page.locator("select").first();
    await templateSelect.selectOption({ label: `Template: ${templateMod.name}` });
    await page.getByPlaceholder("Module name").press("Enter");
    await page.waitForTimeout(500);

    // New module should appear
    await expect(page.getByText(newModName)).toBeVisible({ timeout: 3000 });
  });
});

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

test.describe("Views", () => {
  test("save and delete a view", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-view-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-view-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-view-${uid()}`);
    await createObject(request, mod.id, `VIEW-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(500);

    // Click "Save As..." to create a view
    await page.getByRole("button", { name: /Save As/ }).click();
    await page.waitForTimeout(300);

    const viewName = `View-${uid()}`;
    await page.getByPlaceholder("View name").fill(viewName);
    await page.getByRole("button", { name: /^Save$/ }).click();
    await page.waitForTimeout(500);

    // View should appear in the select dropdown
    const viewSelect = page.locator("select").first();
    await expect(viewSelect).toContainText(viewName);

    // Select the view to reveal Delete button
    await viewSelect.selectOption({ label: viewName });
    await page.waitForTimeout(300);

    // Delete the view
    await page.getByRole("button", { name: /Delete/ }).click();
    await page.waitForTimeout(300);

    // View should be removed from dropdown
    await expect(viewSelect).not.toContainText(viewName);
  });
});

// ---------------------------------------------------------------------------
// Links Tab CRUD
// ---------------------------------------------------------------------------

test.describe("Links Tab CRUD", () => {
  test("create and delete a link", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-link-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-link-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-link-${uid()}`);
    const obj1 = await createObject(request, mod.id, `LNK-SRC-${uid()}`);
    const obj2 = await createObject(request, mod.id, `LNK-TGT-${uid()}`);
    const lt = await createLinkType(request, `lt-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Links" }).click();
    await page.waitForTimeout(500);

    // The link form has selects in order: Source, LinkType, Target
    // Find selects scoped to the link form
    const form = page.locator("form");
    const selects = form.locator("select");
    // Source select (1st)
    await selects.nth(0).selectOption(obj1.id);
    // Link type select (2nd)
    await selects.nth(1).selectOption(lt.id);
    // Target select (3rd)
    await selects.nth(2).selectOption(obj2.id);

    // Submit via Create Link button
    await page.getByRole("button", { name: "Create Link" }).click();
    await page.waitForTimeout(500);

    // Link should appear in the list
    await expect(page.locator("table tbody tr")).toHaveCount(1, { timeout: 3000 });

    // Delete it
    await page.getByRole("button", { name: /Delete/ }).first().click();
    await page.waitForTimeout(500);
  });
});

// ---------------------------------------------------------------------------
// Baselines Tab Diff
// ---------------------------------------------------------------------------

test.describe("Baselines Tab Diff", () => {
  test("create two baselines and view diff", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-bldiff-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-bldiff-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-bldiff-${uid()}`);
    await createObject(request, mod.id, `BL-OBJ-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);

    // Go to Baselines tab
    await page.getByRole("button", { name: "Baselines" }).click();
    await page.waitForTimeout(300);

    // Create first baseline
    await page.getByPlaceholder("Baseline name").fill("v1.0");
    await page.getByRole("button", { name: /Create/ }).click();
    await page.waitForTimeout(500);

    // First baseline should appear
    await expect(page.getByText("v1.0")).toBeVisible({ timeout: 3000 });

    // Go back to Objects tab, modify an object, then create second baseline
    await page.getByRole("button", { name: "Objects" }).click();
    await page.waitForTimeout(300);

    // Add a new object
    const heading2 = `BL-NEW-${uid()}`;
    await page.getByPlaceholder("Heading (e.g. REQ-001)").fill(heading2);
    await page.getByRole("button", { name: "Add" }).click();
    await page.waitForTimeout(500);

    // Go back to Baselines tab
    await page.getByRole("button", { name: "Baselines" }).click();
    await page.waitForTimeout(300);

    // Create second baseline
    await page.getByPlaceholder("Baseline name").fill("v2.0");
    await page.getByRole("button", { name: /Create/ }).click();
    await page.waitForTimeout(500);

    // Both baselines should appear (use cell locator to avoid matching dropdowns)
    await expect(page.getByRole("cell", { name: "v2.0" })).toBeVisible({ timeout: 3000 });

    // Select baselines in compare dropdowns
    const compareSelects = page.locator("select");
    // Baseline A dropdown — select "v1.0"
    await compareSelects.nth(0).selectOption({ label: "v1.0" });
    // Baseline B dropdown — select "v2.0"
    await compareSelects.nth(1).selectOption({ label: "v2.0" });
    await page.waitForTimeout(300);

    // Click Diff button (now enabled)
    await page.getByRole("button", { name: /Diff/ }).click();
    await page.waitForTimeout(500);

    // Diff view should show added object
    await expect(page.getByText(/added|modified|removed/i)).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Attributes Tab CRUD
// ---------------------------------------------------------------------------

test.describe("Attributes Tab CRUD", () => {
  test("create and delete an attribute definition", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-attr-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-attr-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-attr-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Attributes" }).click();
    await page.waitForTimeout(300);

    // Create attribute definition
    const attrName = `attr-${uid()}`;
    await page.getByPlaceholder("Attribute name").fill(attrName);
    await page.getByRole("button", { name: /Create|Add/ }).click();
    await page.waitForTimeout(500);

    // Should appear in the list
    await expect(page.getByText(attrName)).toBeVisible({ timeout: 3000 });

    // Delete it
    await page.getByRole("button", { name: /Delete/ }).first().click();
    await page.waitForTimeout(500);

    // Should be removed
    await expect(page.getByText(attrName)).not.toBeVisible();
  });
});
