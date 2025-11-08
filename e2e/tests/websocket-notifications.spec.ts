import { test, expect } from '@playwright/test';
import { ApiMocks } from '../fixtures/api-mocks';
import { WebSocketMock } from '../fixtures/websocket-mock';
import { waitForLoadingToComplete, assertWebSocketStatus } from '../utils/test-helpers';

test.describe('WebSocket Notifications', () => {
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

  test('should show connecting status initially', async ({ page }) => {
    // Immediately check status - should be connecting
    await assertWebSocketStatus(page, 'connecting');

    // Wait for connection
    await page.waitForTimeout(200);
    await assertWebSocketStatus(page, 'connected');
  });

  test('should show connected status when WebSocket connects', async ({ page }) => {
    // Wait for connection
    await page.waitForTimeout(200);
    await assertWebSocketStatus(page, 'connected');

    // Verify status text
    const statusElement = page.locator('[data-testid="ws-status"]');
    await expect(statusElement).toContainText('Connected');
  });

  test('should show disconnected status when connection fails', async ({ page }) => {
    // Wait for initial connection
    await page.waitForTimeout(200);
    await assertWebSocketStatus(page, 'connected');

    // Simulate connection failure
    await wsMock.simulateConnectionFailure();

    // Verify status changes to disconnected
    await assertWebSocketStatus(page, 'disconnected');
    await expect(page.locator('[data-testid="ws-status"]')).toContainText('Disconnected');
  });

  test('should implement exponential backoff reconnection', async ({ page }) => {
    // Wait for initial connection
    await page.waitForTimeout(200);
    await assertWebSocketStatus(page, 'connected');

    // Simulate failure
    await wsMock.simulateConnectionFailure();
    await assertWebSocketStatus(page, 'disconnected');

    // Check reconnection attempts with timing
    const startTime = Date.now();

    // Should show connecting status during reconnection
    await assertWebSocketStatus(page, 'connecting');

    // Wait for first reconnection attempt (1 second backoff)
    await page.waitForTimeout(1100);
    await assertWebSocketStatus(page, 'connected');

    // Simulate another failure
    await wsMock.simulateConnectionFailure();
    await assertWebSocketStatus(page, 'disconnected');

    // Wait for second reconnection attempt (2 second backoff)
    await page.waitForTimeout(2100);
    await assertWebSocketStatus(page, 'connected');
  });

  test('should handle rapid connect/disconnect cycles', async ({ page }) => {
    // Wait for initial connection
    await page.waitForTimeout(200);
    await assertWebSocketStatus(page, 'connected');

    // Simulate rapid failures
    for (let i = 0; i < 3; i++) {
      await wsMock.simulateConnectionFailure();
      await assertWebSocketStatus(page, 'disconnected');

      // Wait for reconnection
      await page.waitForTimeout(1000 + (i * 1000)); // Increasing backoff
      await assertWebSocketStatus(page, 'connected');
    }

    // Verify system remains stable after multiple failures
    await page.waitForTimeout(1000);
    await assertWebSocketStatus(page, 'connected');
  });

  test('should deliver real-time messages successfully', async ({ page }) => {
    // Wait for connection
    await page.waitForTimeout(200);
    await assertWebSocketStatus(page, 'connected');

    // Send a test message
    const testMessage = JSON.stringify({
      type: 'test_message',
      data: { message: 'Hello from WebSocket!' }
    });

    await wsMock.simulateMessage(testMessage);

    // Verify message is displayed (assuming UI shows WebSocket messages)
    await expect(page.locator('[data-testid="ws-messages"]')).toContainText('Hello from WebSocket!');
  });

  test('should queue messages when disconnected and deliver on reconnect', async ({ page }) => {
    // Wait for connection
    await page.waitForTimeout(200);
    await assertWebSocketStatus(page, 'connected');

    // Simulate disconnection
    await wsMock.simulateConnectionFailure();
    await assertWebSocketStatus(page, 'disconnected');

    // Send message while disconnected (should be queued or handled gracefully)
    const queuedMessage = JSON.stringify({
      type: 'queued_message',
      data: { message: 'This should be queued' }
    });

    await wsMock.simulateMessage(queuedMessage);

    // Reconnect
    await page.waitForTimeout(1100); // Wait for reconnection
    await assertWebSocketStatus(page, 'connected');

    // Verify message is delivered after reconnect
    await expect(page.locator('[data-testid="ws-messages"]')).toContainText('This should be queued');
  });

  test('should show connection status in UI consistently', async ({ page }) => {
    // Check that status indicator is always visible
    await expect(page.locator('[data-testid="ws-status"]')).toBeVisible();

    // Test all states
    const states = ['connecting', 'connected', 'disconnected'];

    for (const state of states) {
      if (state === 'connecting') {
        // Initially connecting
        await assertWebSocketStatus(page, state);
      } else if (state === 'connected') {
        await page.waitForTimeout(200);
        await assertWebSocketStatus(page, state);
      } else if (state === 'disconnected') {
        await wsMock.simulateConnectionFailure();
        await assertWebSocketStatus(page, state);
      }

      // Verify status text matches state
      const statusElement = page.locator('[data-testid="ws-status"]');
      const expectedText = state.charAt(0).toUpperCase() + state.slice(1);
      await expect(statusElement).toContainText(expectedText);
    }
  });

  test('should handle WebSocket errors gracefully', async ({ page }) => {
    // Wait for connection
    await page.waitForTimeout(200);
    await assertWebSocketStatus(page, 'connected');

    // Simulate error
    await page.evaluate(() => {
      const mockWS = (window as any).WebSocket;
      if (mockWS.prototype && mockWS.prototype.onerror) {
        mockWS.prototype.onerror(new Event('error'));
      }
    });

    // Verify error is handled without crashing the app
    await expect(page.locator('[data-testid="ws-status"]')).toBeVisible();
    await expect(page.locator('body')).not.toContainText('Error');
  });
});