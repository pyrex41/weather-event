import { test, expect } from '@playwright/test';

test('basic test', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('[data-testid="nav-dashboard"]')).toBeVisible();
  await expect(page.locator('[data-testid="nav-bookings"]')).toBeVisible();
  await expect(page.locator('[data-testid="nav-students"]')).toBeVisible();
  await expect(page.locator('[data-testid="nav-alerts"]')).toBeVisible();
});