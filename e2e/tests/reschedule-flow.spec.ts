import { test, expect } from '@playwright/test';
import { ApiMocks } from '../fixtures/api-mocks';
import { WebSocketMock } from '../fixtures/websocket-mock';
import { waitForLoadingToComplete } from '../utils/test-helpers';

test.describe('Reschedule Flow', () => {
  let apiMocks: ApiMocks;
  let wsMock: WebSocketMock;

  test.beforeEach(async ({ page }) => {
    apiMocks = new ApiMocks(page);
    wsMock = new WebSocketMock(page);

    await apiMocks.setupAllMocks();
    await wsMock.setupWebSocketMock();

    await page.goto('/');
    await waitForLoadingToComplete(page);
  });

  test('should display 3 AI-suggested reschedule options', async ({ page }) => {
    // Navigate to bookings page
    await page.click('[data-testid="nav-bookings"]');

    // Find a booking and click reschedule
    await page.click('[data-testid="booking-item"]:first-child [data-testid="reschedule-btn"]');

    // Verify reschedule modal appears
    await expect(page.locator('[data-testid="reschedule-modal"]')).toBeVisible();

    // Verify 3 options are displayed
    const options = page.locator('[data-testid="reschedule-option"]');
    await expect(options).toHaveCount(3);

    // Verify each option has required information
    for (let i = 0; i < 3; i++) {
      const option = options.nth(i);
      await expect(option.locator('[data-testid="option-time"]')).toBeVisible();
      await expect(option.locator('[data-testid="option-reason"]')).toBeVisible();
    }
  });

  test('should show instructor availability badges on reschedule options', async ({ page }) => {
    // Navigate to bookings and open reschedule
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="booking-item"]:first-child [data-testid="reschedule-btn"]');

    // Check availability badges
    const options = page.locator('[data-testid="reschedule-option"]');

    // First option should be available
    await expect(options.nth(0).locator('[data-testid="availability-badge"]')).toContainText('Available');
    await expect(options.nth(0).locator('[data-testid="availability-badge"]')).toHaveClass(/available/);

    // Third option should be unavailable
    await expect(options.nth(2).locator('[data-testid="availability-badge"]')).toContainText('Unavailable');
    await expect(options.nth(2).locator('[data-testid="availability-badge"]')).toHaveClass(/unavailable/);
  });

  test('should allow selection of reschedule option and update booking', async ({ page }) => {
    // Navigate to bookings and open reschedule
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="booking-item"]:first-child [data-testid="reschedule-btn"]');

    // Select first available option
    await page.click('[data-testid="reschedule-option"]:first-child [data-testid="select-option-btn"]');

    // Verify confirmation dialog
    await expect(page.locator('[data-testid="confirm-reschedule-modal"]')).toBeVisible();

    // Confirm reschedule
    await page.click('[data-testid="confirm-reschedule-btn"]');

    // Verify success message
    await expect(page.locator('[data-testid="success-message"]')).toContainText('Booking rescheduled successfully');

    // Verify booking time updated in list
    await expect(page.locator('[data-testid="booking-list"]')).toContainText('14:00');
  });

  test('should prevent selection of unavailable instructor slots', async ({ page }) => {
    // Navigate to bookings and open reschedule
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="booking-item"]:first-child [data-testid="reschedule-btn"]');

    // Try to select unavailable option
    const unavailableOption = page.locator('[data-testid="reschedule-option"]').nth(2);
    await expect(unavailableOption.locator('[data-testid="select-option-btn"]')).toBeDisabled();

    // Verify disabled styling
    await expect(unavailableOption.locator('[data-testid="select-option-btn"]')).toHaveClass(/disabled/);
  });

  test('should show weather suitability indicators', async ({ page }) => {
    // Navigate to bookings and open reschedule
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="booking-item"]:first-child [data-testid="reschedule-btn"]');

    const options = page.locator('[data-testid="reschedule-option"]');

    // All options should show weather suitable (based on our mock data)
    for (let i = 0; i < 3; i++) {
      await expect(options.nth(i).locator('[data-testid="weather-indicator"]')).toContainText('Weather OK');
      await expect(options.nth(i).locator('[data-testid="weather-indicator"]')).toHaveClass(/suitable/);
    }
  });

  test('should handle reschedule API errors gracefully', async ({ page }) => {
    // Mock API error for reschedule
    await page.route('**/api/bookings/*/reschedule', async (route) => {
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Internal server error' })
      });
    });

    // Navigate to bookings and attempt reschedule
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="booking-item"]:first-child [data-testid="reschedule-btn"]');
    await page.click('[data-testid="reschedule-option"]:first-child [data-testid="select-option-btn"]');
    await page.click('[data-testid="confirm-reschedule-btn"]');

    // Verify error message
    await expect(page.locator('[data-testid="error-message"]')).toContainText('Failed to reschedule booking');

    // Verify modal remains open
    await expect(page.locator('[data-testid="confirm-reschedule-modal"]')).toBeVisible();
  });

  test('should show loading state during reschedule operation', async ({ page }) => {
    // Navigate to bookings and start reschedule
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="booking-item"]:first-child [data-testid="reschedule-btn"]');
    await page.click('[data-testid="reschedule-option"]:first-child [data-testid="select-option-btn"]');
    await page.click('[data-testid="confirm-reschedule-btn"]');

    // Verify loading state
    await expect(page.locator('[data-testid="reschedule-loading"]')).toBeVisible();
    await expect(page.locator('[data-testid="confirm-reschedule-btn"]')).toBeDisabled();

    // Wait for completion
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();

    // Verify loading state is gone
    await expect(page.locator('[data-testid="reschedule-loading"]')).not.toBeVisible();
  });

  test('should allow cancellation of reschedule operation', async ({ page }) => {
    // Navigate to bookings and open reschedule
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="booking-item"]:first-child [data-testid="reschedule-btn"]');

    // Select option
    await page.click('[data-testid="reschedule-option"]:first-child [data-testid="select-option-btn"]');

    // Cancel instead of confirm
    await page.click('[data-testid="cancel-reschedule-btn"]');

    // Verify modal closes
    await expect(page.locator('[data-testid="confirm-reschedule-modal"]')).not.toBeVisible();

    // Verify booking was not changed
    await expect(page.locator('[data-testid="booking-list"]')).not.toContainText('14:00');
  });
});