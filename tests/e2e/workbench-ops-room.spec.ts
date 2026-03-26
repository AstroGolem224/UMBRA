import { expect, test } from "@playwright/test";

test("workbench supports retry and paged history in browser mode", async ({ page }) => {
  await page.goto("/#/workbench");
  const timeline = page.locator(".timeline-list");

  await expect(page.getByRole("heading", { name: "Workbench" })).toBeVisible();
  await expect(page.getByText("recent dispatches")).toBeVisible();
  await expect(page.getByText("onboarding")).toBeVisible();
  await expect(page.getByRole("button", { name: "checklist", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "retry", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "load older events", exact: true })).toBeVisible();

  await page.getByRole("button", { name: "load older events", exact: true }).click();
  await expect(timeline.locator(".timeline-body").filter({ hasText: "implement the workbench retry flow" })).toBeVisible();

  await page.getByRole("button", { name: "retry", exact: true }).click();
  await expect(page.getByText("reply to run-1")).toBeVisible();
  await expect(timeline.locator(".timeline-body").filter({ hasText: "continued from run run-1" })).toBeVisible();
});

test("ops room renders channel history and paged messages in browser mode", async ({ page }) => {
  await page.goto("/#/ops-room");
  const messageList = page.locator(".message-list");
  const jobList = page.locator(".rail-section").nth(3).locator(".simple-list");

  await expect(page.getByRole("heading", { name: "Ops Room" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "delivery", exact: true })).toBeVisible();
  await expect(page.getByText("onboarding")).toBeVisible();
  await expect(page.getByRole("button", { name: "load older messages", exact: true })).toBeVisible();

  await page.getByRole("button", { name: "load older messages", exact: true }).click();
  await expect(messageList.locator(".message-body").filter({ hasText: "@forge please finish phase 6 and report back" })).toBeVisible();
  await expect(messageList.locator(".message-body").filter({ hasText: "the retry surface is in place; recovery now writes audit events." })).toBeVisible();
  await expect(jobList.getByText("finish phase 6", { exact: true })).toBeVisible();

  await page.getByRole("button", { name: "reply", exact: true }).first().click();
  await expect(page.getByText("replying to")).toBeVisible();
});
