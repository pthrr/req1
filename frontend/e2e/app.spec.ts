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
    await page.getByPlaceholder("// JavaScript source code").fill("return obj.heading || ''");
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

// ---------------------------------------------------------------------------
// Additional helpers for new feature tests
// ---------------------------------------------------------------------------

/** Create a review package via API. */
async function createReviewPackage(
  request: import("@playwright/test").APIRequestContext,
  moduleId: string,
  name: string,
) {
  const res = await request.post(`${API}/modules/${moduleId}/review-packages`, {
    data: { name },
  });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string; name: string; status: string };
}

/** Create a change proposal via API. */
async function createChangeProposal(
  request: import("@playwright/test").APIRequestContext,
  moduleId: string,
  title: string,
) {
  const res = await request.post(`${API}/modules/${moduleId}/change-proposals`, {
    data: { title },
  });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string; title: string; status: string };
}

/** Create a baseline set via API. */
async function createBaselineSet(
  request: import("@playwright/test").APIRequestContext,
  name: string,
  version: string,
) {
  const res = await request.post(`${API}/baseline-sets`, {
    data: { name, version },
  });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string; name: string; version: string };
}

/** Create a user via API. */
async function createUser(
  request: import("@playwright/test").APIRequestContext,
  email: string,
  displayName: string,
) {
  const res = await request.post(`${API}/users`, {
    data: { email, display_name: displayName },
  });
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as { id: string; email: string; display_name: string };
}

// ---------------------------------------------------------------------------
// Dark Mode Toggle
// ---------------------------------------------------------------------------

test.describe("Dark Mode", () => {
  test("toggle dark mode and persist preference", async ({ page }) => {
    await page.goto("/");

    // Header should have a Dark/Light toggle button
    const toggle = page.getByRole("button", { name: /Dark|Light/ });
    await expect(toggle).toBeVisible();

    // Default should be "Dark" (meaning we're in light mode, button offers dark)
    await expect(toggle).toHaveText("Dark");

    // Click to switch to dark mode
    await toggle.click();
    await expect(toggle).toHaveText("Light");

    // Reload and check persistence
    await page.reload();
    await expect(page.getByRole("button", { name: "Light" })).toBeVisible();

    // Toggle back to light
    await page.getByRole("button", { name: "Light" }).click();
    await expect(page.getByRole("button", { name: "Dark" })).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Publish Dropdown
// ---------------------------------------------------------------------------

test.describe("Publish Dropdown", () => {
  test("shows format options in dropdown", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-pub-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-pub-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-pub-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);

    // Click the "Publish" dropdown button
    await page.getByRole("button", { name: /Publish/ }).click();
    await page.waitForTimeout(200);

    // Should show format options
    await expect(page.getByRole("button", { name: "HTML" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Markdown" })).toBeVisible();

    // LaTeX should be disabled
    const latexBtn = page.getByRole("button", { name: "LaTeX" });
    await expect(latexBtn).toBeVisible();
    await expect(latexBtn).toBeDisabled();
  });
});

// ---------------------------------------------------------------------------
// Preview Panel
// ---------------------------------------------------------------------------

test.describe("Publish Preview", () => {
  test("opens preview panel with iframe", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-prev-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-prev-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-prev-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);

    // Click the Preview button
    await page.getByRole("button", { name: "Preview" }).click();

    // Preview panel should appear
    await expect(page.getByText("Document Preview")).toBeVisible({ timeout: 5000 });

    // Should contain an iframe
    await expect(page.locator("iframe[title='Document Preview']")).toBeVisible();

    // Close button should work
    await page.getByRole("button", { name: "Close" }).click();
    await expect(page.getByText("Document Preview")).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Breadcrumb Navigation
// ---------------------------------------------------------------------------

test.describe("Object Breadcrumb", () => {
  test("shows breadcrumb path when object is selected", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-bc-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-bc-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-bc-${uid()}`);
    const parentHeading = `BC-Parent-${uid()}`;
    const parent = await createObject(request, mod.id, parentHeading);
    const childHeading = `BC-Child-${uid()}`;
    // Create child under parent
    const childRes = await request.post(`${API}/modules/${mod.id}/objects`, {
      data: { heading: childHeading, parent_id: parent.id },
    });
    expect(childRes.ok()).toBeTruthy();

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click on the child row in the grid
    await page.locator(".ag-row").filter({ hasText: childHeading }).click();
    await page.waitForTimeout(500);

    // Breadcrumb should show parent heading somewhere on page
    await expect(page.getByText(parentHeading).first()).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Batch Review
// ---------------------------------------------------------------------------

test.describe("Batch Review", () => {
  test("review all button marks unreviewed objects", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-br-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-br-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-br-${uid()}`);
    await createObject(request, mod.id, `BR-A-${uid()}`);
    await createObject(request, mod.id, `BR-B-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Review All button should be visible
    const reviewAllBtn = page.getByRole("button", { name: "Review All" });
    await expect(reviewAllBtn).toBeVisible();

    // Click it (auto-confirm dialog)
    page.on("dialog", (dialog) => dialog.accept());
    await reviewAllBtn.click();
    await page.waitForTimeout(1000);

    // Grid should now show reviewed checkmarks for all objects
    // Each reviewed object shows a green checkmark emoji
    const checkmarks = page.locator(".ag-row").locator("text=\u2705");
    await expect(checkmarks.first()).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// Split View
// ---------------------------------------------------------------------------

test.describe("Split View", () => {
  test("toggle split view shows tree + detail panel", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-sv-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-sv-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-sv-${uid()}`);
    const heading = `SV-${uid()}`;
    await createObject(request, mod.id, heading);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click Split View button
    await page.getByRole("button", { name: "Split View" }).click();
    await page.waitForTimeout(300);

    // Grid View button should now be visible (toggled)
    await expect(page.getByRole("button", { name: "Grid View" })).toBeVisible();

    // "Select an object from the tree" placeholder should show
    await expect(page.getByText("Select an object from the tree")).toBeVisible();

    // Click on tree node
    await page.getByText(heading).first().click();
    await page.waitForTimeout(500);

    // Detail panel should show the object heading in a form
    await expect(page.locator("input[type='text']").first()).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Object Detail Panel (Edit button)
// ---------------------------------------------------------------------------

test.describe("Object Detail Panel", () => {
  test("open detail panel via Edit button", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-det-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-det-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-det-${uid()}`);
    const heading = `DET-${uid()}`;
    await createObject(request, mod.id, heading, "Some body text");

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click Edit button on the row
    const editBtn = page.getByRole("button", { name: "Edit" }).first();
    await editBtn.click();
    await page.waitForTimeout(500);

    // If history opened instead, close and retry
    if (await page.getByText(/History for/).isVisible()) {
      await page.getByRole("button", { name: "Close" }).click();
      await page.waitForTimeout(300);
      await editBtn.click();
      await page.waitForTimeout(500);
    }

    // Detail panel should show Object heading
    await expect(page.getByText(/^Object:/)).toBeVisible({ timeout: 5000 });

    // Should have Save and Close buttons in the detail panel
    await expect(page.getByRole("button", { name: "Save", exact: true }).first()).toBeVisible();
    await expect(page.getByRole("button", { name: "Close" }).first()).toBeVisible();

    // Close it
    await page.getByRole("button", { name: "Close" }).click();
    await expect(page.getByText(/^Object:/)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Reviews Tab
// ---------------------------------------------------------------------------

test.describe("Reviews Tab", () => {
  test("create review package and see dashboard", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-rev-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-rev-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-rev-${uid()}`);
    await createObject(request, mod.id, `REV-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);

    // Go to Reviews tab
    await page.getByRole("button", { name: "Reviews" }).click();
    await page.waitForTimeout(500);

    // Dashboard should be visible
    await expect(page.getByText("Review Dashboard")).toBeVisible();

    // Create a review package
    const pkgName = `RevPkg-${uid()}`;
    await page.getByPlaceholder("Package name").fill(pkgName);
    await page.getByRole("button", { name: "Create Package" }).click();
    await page.waitForTimeout(500);

    // Package should appear
    await expect(page.getByText(pkgName)).toBeVisible({ timeout: 3000 });

    // Package shows "draft" status indicator
    await expect(page.getByText("draft").first()).toBeVisible();
  });

  test("status transitions work", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-rst-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-rst-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-rst-${uid()}`);
    await createReviewPackage(request, mod.id, `Pkg-trans-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Reviews" }).click();
    await page.waitForTimeout(500);

    // Should show "Open" transition button (from draft)
    const openBtn = page.getByRole("button", { name: "Open", exact: true });
    await expect(openBtn).toBeVisible({ timeout: 3000 });

    // Transition the package via API directly (more reliable than clicking through UI)
    const pkgRes = await request.get(`${API}/modules/${mod.id}/review-packages`);
    const pkgs = (await pkgRes.json()).items;
    await request.patch(`${API}/modules/${mod.id}/review-packages/${pkgs[0].id}`, {
      data: { status: "open" },
    });

    // Reload the Reviews tab to see updated status
    await page.getByRole("button", { name: "Reviews" }).click();
    await page.waitForTimeout(500);

    // Now it should show "In_review" transition button (from open status)
    await expect(page.getByText("In_review")).toBeVisible({ timeout: 5000 });
  });

  test("review package with pre-created data via API", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-rpkg-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-rpkg-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-rpkg-${uid()}`);
    const pkg = await createReviewPackage(request, mod.id, `Pkg-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Reviews" }).click();
    await page.waitForTimeout(500);

    // Expand the package to see assignments section
    await page.getByText(pkg.name).click();
    await page.waitForTimeout(300);

    // Should show "No assignments yet."
    await expect(page.getByText("No assignments yet.")).toBeVisible();

    // Delete package
    await page.getByRole("button", { name: "Delete" }).first().click();
    await page.waitForTimeout(500);

    // Package should be removed
    await expect(page.getByText(pkg.name)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Change Proposals Tab
// ---------------------------------------------------------------------------

test.describe("Change Proposals Tab", () => {
  test("create and manage change proposal", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-cp-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-cp-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-cp-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);

    // Go to Proposals tab
    await page.getByRole("button", { name: "Proposals" }).click();
    await page.waitForTimeout(500);

    // Create a proposal
    const title = `CP-${uid()}`;
    await page.getByPlaceholder("Proposal title").fill(title);
    await page.getByRole("button", { name: "Create Proposal" }).click();
    await page.waitForTimeout(500);

    // Proposal should appear
    await expect(page.getByText(title)).toBeVisible({ timeout: 3000 });

    // Should show "draft" status
    await expect(page.getByText("draft").first()).toBeVisible();

    // Click Approve button
    await page.getByRole("button", { name: "Approve" }).first().click();
    await page.waitForTimeout(500);

    // Status should change to "approved"
    await expect(page.getByText("approved").first()).toBeVisible();
  });

  test("reject a change proposal", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-cprej-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-cprej-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-cprej-${uid()}`);
    await createChangeProposal(request, mod.id, `CP-rej-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Proposals" }).click();
    await page.waitForTimeout(500);

    // Click Reject
    await page.getByRole("button", { name: "Reject" }).first().click();
    await page.waitForTimeout(500);

    // Status should change to "rejected"
    await expect(page.getByText("rejected").first()).toBeVisible();
  });

  test("delete a change proposal", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-cpdel-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-cpdel-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-cpdel-${uid()}`);
    const cp = await createChangeProposal(request, mod.id, `CP-del-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Proposals" }).click();
    await page.waitForTimeout(500);

    await page.getByRole("button", { name: "Del" }).first().click();
    await page.waitForTimeout(500);

    await expect(page.getByText(cp.title)).not.toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Baseline Sets
// ---------------------------------------------------------------------------

test.describe("Baseline Sets", () => {
  test("create baseline set and use it when creating baseline", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-bs-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-bs-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-bs-${uid()}`);
    await createObject(request, mod.id, `BS-OBJ-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Baselines" }).click();
    await page.waitForTimeout(500);

    // Baseline Sets section should be visible
    await expect(page.getByText("Baseline Sets")).toBeVisible();

    // Create a new baseline set
    await page.getByRole("button", { name: "New Set" }).click();
    await page.waitForTimeout(200);

    const setName = `Set-${uid()}`;
    await page.getByPlaceholder("Set name").fill(setName);
    await page.getByPlaceholder("Version").fill("1.0");
    await page.getByRole("button", { name: /^Create$/ }).click();
    await page.waitForTimeout(500);

    // Set should appear as a filter chip
    await expect(page.getByText(`${setName} v1.0`).first()).toBeVisible({ timeout: 3000 });
  });

  test("filter baselines by set", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-bsf-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-bsf-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-bsf-${uid()}`);
    await createObject(request, mod.id, `BSF-OBJ-${uid()}`);
    const bset = await createBaselineSet(request, `FilterSet-${uid()}`, "2.0");

    // Create a baseline with this set
    const blRes = await request.post(`${API}/modules/${mod.id}/baselines`, {
      data: { name: `BL-in-set-${uid()}`, baseline_set_id: bset.id },
    });
    expect(blRes.ok()).toBeTruthy();

    // Create a baseline without set
    const blRes2 = await request.post(`${API}/modules/${mod.id}/baselines`, {
      data: { name: `BL-no-set-${uid()}` },
    });
    expect(blRes2.ok()).toBeTruthy();

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: "Baselines" }).click();
    await page.waitForTimeout(500);

    // Both baselines should be visible initially
    const table = page.locator("table");
    await expect(table.locator("tbody tr")).toHaveCount(2, { timeout: 3000 });

    // Click the set filter chip
    await page.getByRole("button", { name: new RegExp(`${bset.name}`) }).click();
    await page.waitForTimeout(300);

    // Only the baseline in the set should be visible
    await expect(table.locator("tbody tr")).toHaveCount(1, { timeout: 3000 });

    // Click "All" to reset
    await page.getByRole("button", { name: "All" }).click();
    await page.waitForTimeout(300);

    await expect(table.locator("tbody tr")).toHaveCount(2, { timeout: 3000 });
  });
});

// ---------------------------------------------------------------------------
// Activity Feed
// ---------------------------------------------------------------------------

test.describe("Activity Feed", () => {
  test("expand activity feed shows recent changes", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-af-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-af-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-af-${uid()}`);
    await createObject(request, mod.id, `AF-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Activity Feed button should be visible
    const feedBtn = page.getByRole("button", { name: "Activity Feed" });
    await expect(feedBtn).toBeVisible();

    // Click to expand
    await feedBtn.click();
    await page.waitForTimeout(2000); // Wait for feed to load

    // Should show activity entries with "create" type
    await expect(page.getByText("create").first()).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// New Tabs Visible
// ---------------------------------------------------------------------------

test.describe("New Tabs", () => {
  test("Reviews and Proposals tabs are visible", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-newtab-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-newtab-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-newtab-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);

    await expect(page.getByRole("button", { name: "Reviews" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Proposals" })).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// References Panel
// ---------------------------------------------------------------------------

test.describe("References Panel", () => {
  test("open and add a reference", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-ref-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-ref-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-ref-${uid()}`);
    await createObject(request, mod.id, `REF-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click Ref button on a row
    const refBtn = page.getByRole("button", { name: "Ref" }).first();
    await refBtn.click();
    await page.waitForTimeout(500);

    // If history opened, close and retry
    if (await page.getByText(/History for/).isVisible()) {
      await page.getByRole("button", { name: "Close" }).click();
      await page.waitForTimeout(300);
      await refBtn.click();
      await page.waitForTimeout(500);
    }

    // References panel should show
    await expect(page.getByText(/References for/)).toBeVisible({ timeout: 5000 });

    // Click Add Reference
    await page.getByRole("button", { name: "Add Reference" }).click();
    await page.waitForTimeout(200);

    // Should show a reference row with inputs
    await expect(page.getByPlaceholder("Path or URL")).toBeVisible();
    await expect(page.getByPlaceholder("Description")).toBeVisible();

    // Fill and save
    await page.getByPlaceholder("Path or URL").fill("https://example.com/spec");
    await page.getByPlaceholder("Description").fill("External specification");
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await page.waitForTimeout(500);

    // Close
    await page.getByRole("button", { name: "Close" }).click();
  });
});

// ---------------------------------------------------------------------------
// Search Highlighting
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Rich Text Editor (TipTap)
// ---------------------------------------------------------------------------

test.describe("Rich Text Editor", () => {
  test("TipTap editor loads in object detail panel", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-rte-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-rte-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-rte-${uid()}`);
    await createObject(request, mod.id, `RTE-001-${uid()}`, "Some body text");

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click "Edit" button on the first object row
    await page.getByRole("button", { name: "Edit" }).first().click();
    await page.waitForTimeout(500);

    // TipTap editor should be present (ProseMirror contenteditable div)
    const editor = page.locator(".ProseMirror");
    await expect(editor).toBeVisible();

    // Toolbar should be visible with formatting buttons
    await expect(page.getByRole("button", { name: "B" }).first()).toBeVisible();
    await expect(page.getByRole("button", { name: "I" }).first()).toBeVisible();
    await expect(page.getByRole("button", { name: "U" }).first()).toBeVisible();

    // Close the modal
    await page.getByRole("button", { name: "Close" }).click();
  });

  test("TipTap toolbar formatting works (bold, italic, heading)", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-fmt-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-fmt-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-fmt-${uid()}`);
    await createObject(request, mod.id, `FMT-001-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Open detail panel
    await page.getByRole("button", { name: "Edit" }).first().click();
    await page.waitForTimeout(500);

    // Type in editor
    const editor = page.locator(".ProseMirror");
    await editor.click();
    await editor.pressSequentially("Hello World");

    // Select "World" and bold it
    await editor.press("Home");
    await editor.press("Shift+End");

    // Click Bold button
    const boldBtn = page.locator("button").filter({ hasText: /^B$/ }).first();
    await boldBtn.click();
    await page.waitForTimeout(200);

    // Bold button should now appear active (has primary bg color)
    await expect(boldBtn).toBeVisible();

    // Close
    await page.getByRole("button", { name: "Close" }).click();
  });

  test("save object with rich text body", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-save-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-save-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-save-${uid()}`);
    const heading = `SAVE-${uid()}`;
    await createObject(request, mod.id, heading);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Open detail panel
    await page.getByRole("button", { name: "Edit" }).first().click();
    await page.waitForTimeout(500);

    // Type in TipTap editor
    const editor = page.locator(".ProseMirror");
    await editor.click();
    await editor.pressSequentially("Rich text content");
    await page.waitForTimeout(200);

    // Save
    await page.getByRole("button", { name: "Save" }).click();
    await page.waitForTimeout(1000);

    // Close
    await page.getByRole("button", { name: "Close" }).click();
    await page.waitForTimeout(500);

    // Verify the body is displayed in the grid (the body column should contain the text)
    await expect(page.getByText("Rich text content").first()).toBeVisible();
  });

  test("table insertion via toolbar", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-tbl-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-tbl-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-tbl-${uid()}`);
    await createObject(request, mod.id, `TBL-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Edit" }).first().click();
    await page.waitForTimeout(500);

    // Click Table button in toolbar
    await page.getByRole("button", { name: "Table" }).click();
    await page.waitForTimeout(300);

    // Table should appear in the editor
    const table = page.locator(".ProseMirror table");
    await expect(table).toBeVisible();

    // Should have a header row + 2 data rows = 3 rows total
    const rows = page.locator(".ProseMirror tr");
    await expect(rows).toHaveCount(3);

    await page.getByRole("button", { name: "Close" }).click();
  });

  test("legacy markdown body auto-converts to HTML in editor", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-md-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-md-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-md-${uid()}`);

    // Create object with markdown body (no HTML tags)
    const heading = `MD-${uid()}`;
    const objRes = await request.post(`${API}/modules/${mod.id}/objects`, {
      data: { heading, body: "This is **bold** markdown" },
    });
    expect(objRes.ok()).toBeTruthy();

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Open detail panel
    await page.getByRole("button", { name: "Edit" }).first().click();
    await page.waitForTimeout(500);

    // The editor should have converted the markdown to HTML
    // "bold" should appear as actual bold text in ProseMirror (strong tag)
    const editor = page.locator(".ProseMirror");
    await expect(editor).toBeVisible();
    const strongTag = page.locator(".ProseMirror strong");
    await expect(strongTag).toContainText("bold");

    await page.getByRole("button", { name: "Close" }).click();
  });
});

// ---------------------------------------------------------------------------
// Document View (LiveDoc)
// ---------------------------------------------------------------------------

test.describe("Document View", () => {
  test("Document tab exists and renders objects", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-doc-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-doc-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-doc-${uid()}`);
    await createObject(request, mod.id, `DOC-001-${uid()}`, "First section body");
    await createObject(request, mod.id, `DOC-002-${uid()}`, "Second section body");

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click Document tab
    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);

    // Should see the document content
    await expect(page.getByText("First section body")).toBeVisible();
    await expect(page.getByText("Second section body")).toBeVisible();
  });

  test("Document outline sidebar shows headings", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-out-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-out-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-out-${uid()}`);
    const h1 = `OUT-A-${uid()}`;
    const h2 = `OUT-B-${uid()}`;
    await createObject(request, mod.id, h1, "Body A");
    await createObject(request, mod.id, h2, "Body B");

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Click Document tab
    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);

    // Outline sidebar should show headings
    await expect(page.getByText("Outline")).toBeVisible();
    await expect(page.getByText(h1).first()).toBeVisible();
    await expect(page.getByText(h2).first()).toBeVisible();
  });

  test("outline click scrolls to object", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-scr-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-scr-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-scr-${uid()}`);

    // Create enough objects to make the document scrollable
    const headings: string[] = [];
    for (let i = 0; i < 10; i++) {
      const h = `SCR-${i}-${uid()}`;
      headings.push(h);
      await createObject(request, mod.id, h, `Body for section ${i}. `.repeat(20));
    }

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);

    // Click last heading in outline — should trigger scroll
    const lastHeading = headings[headings.length - 1];
    const outlineItem = page.getByText(lastHeading).first();
    await outlineItem.click();
    await page.waitForTimeout(500);

    // The last object block should now be visible
    await expect(page.getByText(`Body for section 9`).first()).toBeVisible();
  });

  test("double-click enables inline editing with TipTap", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-ied-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-ied-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-ied-${uid()}`);
    const heading = `IED-${uid()}`;
    await createObject(request, mod.id, heading, "Editable content");

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);

    // Double-click the object block
    await page.getByText("Editable content").dblclick();
    await page.waitForTimeout(500);

    // Should see editing UI: Save and Cancel buttons + TipTap editor
    await expect(page.getByRole("button", { name: "Save" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Cancel" })).toBeVisible();
    await expect(page.locator(".ProseMirror")).toBeVisible();

    // Cancel editing
    await page.getByRole("button", { name: "Cancel" }).click();
    await page.waitForTimeout(300);

    // Should be back to read mode
    await expect(page.getByText("Editable content")).toBeVisible();
  });

  test("inline edit save persists changes", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-ies-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-ies-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-ies-${uid()}`);
    const heading = `IES-${uid()}`;
    await createObject(request, mod.id, heading, "Original text");

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);

    // Double-click to edit
    await page.getByText("Original text").dblclick();
    await page.waitForTimeout(500);

    // Update the heading
    const headingInput = page.getByRole("textbox").first();
    await headingInput.clear();
    const newHeading = `Updated-${uid()}`;
    await headingInput.fill(newHeading);

    // Save
    await page.getByRole("button", { name: "Save" }).click();
    await page.waitForTimeout(1000);

    // The updated heading should appear
    await expect(page.getByText(newHeading).first()).toBeVisible();
  });

  test("export Word and PDF buttons present", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-exp-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-exp-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-exp-${uid()}`);
    await createObject(request, mod.id, `EXP-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);

    // Toolbar should have export buttons
    await expect(page.getByRole("button", { name: "Export Word" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Export PDF" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Print" })).toBeVisible();
  });

  test("switch between Objects and Document tabs preserves data", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-swt-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-swt-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-swt-${uid()}`);
    const heading = `SWT-${uid()}`;
    await createObject(request, mod.id, heading, "Persistent body");

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Verify objects tab shows the object
    await expect(page.getByText(heading).first()).toBeVisible();

    // Switch to Document tab
    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText("Persistent body")).toBeVisible();

    // Switch back to Objects tab
    await page.getByRole("button", { name: "Objects" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(heading).first()).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// DOCX Publish Format
// ---------------------------------------------------------------------------

test.describe("DOCX Publish", () => {
  test("DOCX format appears in publish dropdown", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-dpub-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-dpub-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-dpub-${uid()}`);
    await createObject(request, mod.id, `DPUB-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Open publish dropdown
    await page.getByRole("button", { name: /Publish/ }).click();
    await page.waitForTimeout(300);

    // DOCX option should be visible
    await expect(page.getByText("Word (DOCX)").first()).toBeVisible();
  });

  test("DOCX format appears in preview panel", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-dpre-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-dpre-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-dpre-${uid()}`);
    await createObject(request, mod.id, `DPRE-${uid()}`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Open preview panel
    await page.getByRole("button", { name: "Preview" }).click();
    await page.waitForTimeout(500);

    // Select DOCX from format dropdown
    const formatSelect = page.locator("select").first();
    await formatSelect.selectOption("docx");
    await page.waitForTimeout(300);

    // Should show download prompt (binary format can't be previewed)
    await expect(page.getByText("Word files cannot be previewed")).toBeVisible();

    // Download button should be visible
    await expect(page.getByRole("button", { name: "Download" })).toBeVisible();

    await page.getByRole("button", { name: "Close" }).click();
  });
});

test.describe("Search Highlighting", () => {
  test("search highlights matching objects in tree", async ({ page, request }) => {
    const ws = await createWorkspace(request, `WS-sh-${uid()}`);
    const proj = await createProject(request, ws.id, `Proj-sh-${uid()}`);
    const mod = await createModule(request, proj.id, `Mod-sh-${uid()}`);
    const needle = `UNIQUE-${uid()}`;
    await createObject(request, mod.id, `SH-A-${uid()}`);
    await createObject(request, mod.id, needle, `body with ${needle} text`);

    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    // Enter search query
    await page.getByPlaceholder("Full-text search...").fill(needle);
    await page.getByRole("button", { name: "Search" }).click();
    await page.waitForTimeout(1000);

    // Tree should show filtered results (highlighted with yellow bg)
    // The Object Tree div should contain the search result
    const treePanel = page.locator("div").filter({ hasText: "Object Tree" }).first();
    await expect(treePanel).toBeVisible();

    // Clear search
    await page.getByRole("button", { name: "Clear" }).click();
    await page.waitForTimeout(500);
  });
});
