import { test, expect } from '@playwright/test';
import { ApiMocks } from '../fixtures/api-mocks';
import { WebSocketMock } from '../fixtures/websocket-mock';
import { TEST_STUDENT, TEST_BOOKING } from '../fixtures/test-data';
import { waitForLoadingToComplete, fillStudentForm, fillBookingForm } from '../utils/test-helpers';

test.describe('Error Scenarios', () => {
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

  test('should handle API 500 errors gracefully', async ({ page }) => {
    // Mock API to return 500 error
    await page.route('**/api/students', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Internal server error' })
        });
      }
    });

    // Try to create student
    await page.click('[data-testid="nav-students"]');
    await page.click('[data-testid="create-student-btn"]');
    await fillStudentForm(page, TEST_STUDENT);
    await page.click('[data-testid="submit-student-btn"]');

    // Verify error message
    await expect(page.locator('[data-testid="error-message"]')).toContainText('Failed to create student');
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
  });

  test('should handle network timeouts', async ({ page }) => {
    // Mock slow API response (timeout)
    await page.route('**/api/bookings', async (route) => {
      // Delay response to simulate timeout
      await new Promise(resolve => setTimeout(resolve, 35000)); // Longer than test timeout
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ bookings: [] })
      });
    }, { timeout: 30000 });

    // Try to load bookings page
    await page.click('[data-testid="nav-bookings"]');

    // Should show timeout error or loading state that eventually fails
    await expect(page.locator('[data-testid="error-message"], [data-testid="timeout-error"]')).toBeVisible();
  });

  test('should handle form validation for invalid coordinates', async ({ page }) => {
    // Navigate to booking creation
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="create-booking-btn"]');

    // Fill form with invalid location
    await page.selectOption('[data-testid="booking-aircraft"]', 'Cessna 172');
    await page.fill('[data-testid="booking-start-time"]', '2025-11-08T10:00:00');
    await page.fill('[data-testid="booking-end-time"]', '2025-11-08T12:00:00');
    await page.fill('[data-testid="booking-location"]', 'INVALID');

    await page.click('[data-testid="submit-booking-btn"]');

    // Should show validation error for invalid location
    await expect(page.locator('[data-testid="error-location"]')).toContainText('Invalid airport code');
  });

  test('should handle WebSocket connection failures', async ({ page }) => {
    // Wait for initial connection
    await page.waitForTimeout(200);

    // Simulate WebSocket connection failure
    await wsMock.simulateConnectionFailure();

    // Verify connection status shows disconnected
    await expect(page.locator('[data-testid="ws-status"]')).toContainText('Disconnected');

    // Verify app continues to function (can still navigate, etc.)
    await page.click('[data-testid="nav-students"]');
    await expect(page.locator('[data-testid="student-list"]')).toBeVisible();
  });

  test('should handle malformed API responses', async ({ page }) => {
    // Mock API to return malformed JSON
    await page.route('**/api/students', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: 'invalid json {{{'
      });
    });

    // Navigate to students page
    await page.click('[data-testid="nav-students"]');

    // Should handle error gracefully
    await expect(page.locator('[data-testid="error-message"]')).toContainText('Failed to load students');
  });

  test('should handle API rate limiting', async ({ page }) => {
    let requestCount = 0;

    // Mock API to return 429 (Too Many Requests) after a few calls
    await page.route('**/api/bookings', async (route) => {
      requestCount++;
      if (requestCount > 3) {
        await route.fulfill({
          status: 429,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Rate limit exceeded' })
        });
      } else {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ bookings: [] })
        });
      }
    });

    // Make multiple rapid requests by navigating quickly
    await page.click('[data-testid="nav-bookings"]');
    await page.waitForTimeout(100);
    await page.click('[data-testid="nav-students"]');
    await page.waitForTimeout(100);
    await page.click('[data-testid="nav-bookings"]');
    await page.waitForTimeout(100);
    await page.click('[data-testid="nav-students"]');

    // Should eventually show rate limit error
    await expect(page.locator('[data-testid="error-message"]')).toContainText('Rate limit exceeded');
  });

  test('should handle concurrent API calls without conflicts', async ({ page }) => {
    // Mock slow API responses
    await page.route('**/api/students', async (route) => {
      await new Promise(resolve => setTimeout(resolve, 1000));
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ students: [] })
      });
    });

    await page.route('**/api/bookings', async (route) => {
      await new Promise(resolve => setTimeout(resolve, 1000));
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ bookings: [] })
      });
    });

    // Trigger multiple concurrent requests
    await page.click('[data-testid="nav-students"]');
    await page.click('[data-testid="nav-bookings"]');

    // Both should complete successfully without interference
    await expect(page.locator('[data-testid="student-list"]')).toBeVisible();
    await expect(page.locator('[data-testid="booking-list"]')).toBeVisible();
  });

  test('should handle invalid form data submission', async ({ page }) => {
    // Navigate to booking creation
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="create-booking-btn"]');

    // Fill form with invalid data types
    await page.fill('[data-testid="booking-start-time"]', 'not-a-date');
    await page.fill('[data-testid="booking-end-time"]', 'also-not-a-date');
    await page.fill('[data-testid="booking-location"]', '123456789012345678901234567890123456789012345678901234567890'); // Too long

    await page.click('[data-testid="submit-booking-btn"]');

    // Should show multiple validation errors
    await expect(page.locator('[data-testid="error-start-time"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-end-time"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-location"]')).toBeVisible();
  });

  test('should recover from temporary API outages', async ({ page }) => {
    let failCount = 0;

    // Mock API that fails temporarily then recovers
    await page.route('**/api/students', async (route) => {
      failCount++;
      if (failCount <= 2) {
        await route.fulfill({
          status: 503,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Service temporarily unavailable' })
        });
      } else {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ students: [] })
        });
      }
    });

    // Navigate to students page (should fail initially)
    await page.click('[data-testid="nav-students"]');
    await expect(page.locator('[data-testid="error-message"]')).toContainText('Service temporarily unavailable');

    // Try again (should succeed)
    await page.click('[data-testid="nav-students"]');
    await expect(page.locator('[data-testid="student-list"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-message"]')).not.toBeVisible();
  });
});