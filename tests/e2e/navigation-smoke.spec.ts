import { expect, test } from "@playwright/test";

const VIEWS = [
  { path: "/#/dashboard", heading: "Umbra Dashboard Overview" },
  { path: "/#/agents", heading: "Agents" },
  { path: "/#/workbench", heading: "Workbench" },
  { path: "/#/ops-room", heading: "Ops Room" },
  { path: "/#/tasks", heading: "Tasks" },
  { path: "/#/cron", heading: "Cron" },
  { path: "/#/notes", heading: "Notes" },
  { path: "/#/launcher", heading: "Launcher" },
  { path: "/#/plugins", heading: "Integrations" },
  { path: "/#/skills", heading: "Skills" },
  { path: "/#/settings", heading: "Settings" },
];

for (const { path, heading } of VIEWS) {
  test(`navigates to ${path} and shows heading`, async ({ page }) => {
    await page.goto(path);
    await expect(
      page.getByRole("heading", { name: heading }).first(),
    ).toBeVisible({ timeout: 5000 });
  });
}

test("sidebar navigation links work", async ({ page }) => {
  await page.goto("/#/dashboard");
  await expect(page.getByRole("heading", { name: "Umbra Dashboard Overview" })).toBeVisible();

  await page.getByRole("link", { name: "Tasks" }).click();
  await expect(page.getByRole("heading", { name: "Tasks" })).toBeVisible();

  await page.getByRole("link", { name: "Notes" }).click();
  await expect(page.getByRole("heading", { name: "Notes" })).toBeVisible();
});

test("command palette opens with Ctrl+K", async ({ page }) => {
  await page.goto("/#/dashboard");
  await page.keyboard.press("Control+k");
  await expect(page.getByPlaceholder("search commands, notes")).toBeVisible();
  await page.keyboard.press("Escape");
});
