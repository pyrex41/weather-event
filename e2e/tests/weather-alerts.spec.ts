import { test, expect } from '@playwright/test';
import { ApiMocks } from '../fixtures/api-mocks';
import { WebSocketMock } from '../fixtures/websocket-mock';
import { waitForLoadingToComplete, assertWeatherAlertVisible, assertDashboardStats } from '../utils/test-helpers';

test.describe('Weather Alerts', () => {
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

  test('should display real-time weather alert banner via WebSocket', async ({ page }) => {
    // Wait for WebSocket connection
    await page.waitForTimeout(200);

    // Simulate severe weather alert
    await wsMock.simulateWeatherAlert('severe');

    // Verify alert banner appears
    await assertWeatherAlertVisible(page, 'Thunderstorm warning');
    await expect(page.locator('[data-testid="weather-alert"]')).toHaveClass(/severe/);

    // Verify alert contains location and timestamp
    await expect(page.locator('[data-testid="weather-alert"]')).toContainText('KORD');
  });

  test('should handle multiple simultaneous weather alerts', async ({ page }) => {
    // Simulate multiple alerts
    await wsMock.simulateWeatherAlert('severe');
    await page.waitForTimeout(100);
    await wsMock.simulateMessage(JSON.stringify({
      type: 'weather_alert',
      data: {
        location: 'KDPA',
        severity: 'moderate',
        description: 'Wind shear warning',
        timestamp: new Date().toISOString()
      }
    }));

    // Verify multiple alerts are displayed
    const alerts = page.locator('[data-testid="weather-alert"]');
    await expect(alerts).toHaveCount(2);

    // Verify different severities are styled differently
    await expect(page.locator('[data-testid="weather-alert"]').first()).toHaveClass(/severe/);
    await expect(page.locator('[data-testid="weather-alert"]').nth(1)).toHaveClass(/moderate/);
  });

  test('should allow weather alert dismissal', async ({ page }) => {
    // Simulate weather alert
    await wsMock.simulateWeatherAlert('severe');

    // Verify alert is visible
    await assertWeatherAlertVisible(page, 'Thunderstorm warning');

    // Click dismiss button
    await page.click('[data-testid="dismiss-alert-btn"]');

    // Verify alert is hidden
    await expect(page.locator('[data-testid="weather-alert"]')).not.toBeVisible();
  });

  test('should update dashboard stats when weather alerts are received', async ({ page }) => {
    // Check initial stats
    await assertDashboardStats(page, { activeAlerts: 0 });

    // Simulate weather alert
    await wsMock.simulateWeatherAlert('severe');

    // Verify stats update
    await assertDashboardStats(page, { activeAlerts: 1 });

    // Simulate another alert
    await wsMock.simulateMessage(JSON.stringify({
      type: 'weather_alert',
      data: {
        location: 'KMDW',
        severity: 'high',
        description: 'Heavy rain',
        timestamp: new Date().toISOString()
      }
    }));

    // Verify stats update again
    await assertDashboardStats(page, { activeAlerts: 2 });

    // Dismiss one alert
    await page.click('[data-testid="dismiss-alert-btn"]');

    // Verify stats update
    await assertDashboardStats(page, { activeAlerts: 1 });
  });

  test('should handle weather alert with clear weather message', async ({ page }) => {
    // First simulate severe weather
    await wsMock.simulateWeatherAlert('severe');
    await assertWeatherAlertVisible(page, 'Thunderstorm warning');

    // Then simulate clear weather
    await wsMock.simulateWeatherAlert('clear');

    // Verify severe alert is replaced or clear message appears
    const alerts = page.locator('[data-testid="weather-alert"]');
    // Either the severe alert should be gone, or a clear message should appear
    const alertText = await alerts.textContent();
    expect(alertText).not.toContain('Thunderstorm warning');
  });

  test('should persist weather alerts across page navigation', async ({ page }) => {
    // Simulate weather alert on dashboard
    await wsMock.simulateWeatherAlert('severe');
    await assertWeatherAlertVisible(page, 'Thunderstorm warning');

    // Navigate to students page
    await page.click('[data-testid="nav-students"]');

    // Verify alert persists
    await assertWeatherAlertVisible(page, 'Thunderstorm warning');

    // Navigate to bookings page
    await page.click('[data-testid="nav-bookings"]');

    // Verify alert still persists
    await assertWeatherAlertVisible(page, 'Thunderstorm warning');
  });

  test('should show appropriate styling for different alert severities', async ({ page }) => {
    // Test severe alert styling
    await wsMock.simulateWeatherAlert('severe');
    await expect(page.locator('[data-testid="weather-alert"]')).toHaveClass(/severe/);

    // Dismiss and test moderate alert
    await page.click('[data-testid="dismiss-alert-btn"]');
    await wsMock.simulateMessage(JSON.stringify({
      type: 'weather_alert',
      data: {
        location: 'KORD',
        severity: 'moderate',
        description: 'Moderate winds',
        timestamp: new Date().toISOString()
      }
    }));
    await expect(page.locator('[data-testid="weather-alert"]')).toHaveClass(/moderate/);

    // Dismiss and test low alert
    await page.click('[data-testid="dismiss-alert-btn"]');
    await wsMock.simulateMessage(JSON.stringify({
      type: 'weather_alert',
      data: {
        location: 'KORD',
        severity: 'low',
        description: 'Light rain',
        timestamp: new Date().toISOString()
      }
    }));
    await expect(page.locator('[data-testid="weather-alert"]')).toHaveClass(/low/);
  });
});