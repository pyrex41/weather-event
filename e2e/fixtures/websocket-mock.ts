// WebSocket mocking utilities for Playwright tests
import { Page } from '@playwright/test';

export class WebSocketMock {
  private page: Page;
  private mockWebSocket: any;
  private messageQueue: string[] = [];
  private connectionState: 'connecting' | 'connected' | 'disconnected' = 'disconnected';

  constructor(page: Page) {
    this.page = page;
  }

  async setupWebSocketMock() {
    // Override the native WebSocket
    await this.page.addInitScript(() => {
      // Store original WebSocket
      const OriginalWebSocket = window.WebSocket;

      // Mock WebSocket class
      class MockWebSocket {
        url: string;
        readyState: number = 0; // CONNECTING
        onopen: ((event: Event) => void) | null = null;
        onmessage: ((event: MessageEvent) => void) | null = null;
        onclose: ((event: CloseEvent) => void) | null = null;
        onerror: ((event: Event) => void) | null = null;

        constructor(url: string) {
          this.url = url;
          // Simulate connection delay
          setTimeout(() => {
            this.readyState = 1; // OPEN
            if (this.onopen) {
              this.onopen(new Event('open'));
            }
          }, 100);
        }

        send(data: string) {
          // Mock send - could trigger responses based on data
          console.log('Mock WebSocket send:', data);
        }

        close() {
          this.readyState = 3; // CLOSED
          if (this.onclose) {
            this.onclose(new CloseEvent('close', { code: 1000, reason: 'Normal closure' }));
          }
        }
      }

      // Replace global WebSocket
      (window as any).WebSocket = MockWebSocket;
    });

    // Store reference for programmatic control
    this.mockWebSocket = await this.page.evaluateHandle(() => (window as any).WebSocket);
  }

  async simulateMessage(message: string) {
    await this.page.evaluate((msg) => {
      // Find the active WebSocket instance and trigger onmessage
      const mockWS = (window as any).WebSocket;
      if (mockWS.prototype && mockWS.prototype.onmessage) {
        const event = new MessageEvent('message', { data: msg });
        mockWS.prototype.onmessage(event);
      }
    }, message);
  }

  async simulateWeatherAlert(alertType: 'severe' | 'clear') {
    const alertMessage = alertType === 'severe'
      ? JSON.stringify({
          type: 'weather_alert',
          data: {
            location: 'KORD',
            severity: 'severe',
            description: 'Thunderstorm warning',
            timestamp: new Date().toISOString()
          }
        })
      : JSON.stringify({
          type: 'weather_clear',
          data: {
            location: 'KORD',
            timestamp: new Date().toISOString()
          }
        });

    await this.simulateMessage(alertMessage);
  }

  async simulateConnectionFailure() {
    await this.page.evaluate(() => {
      const mockWS = (window as any).WebSocket;
      if (mockWS.prototype && mockWS.prototype.onerror) {
        mockWS.prototype.onerror(new Event('error'));
      }
      if (mockWS.prototype && mockWS.prototype.onclose) {
        mockWS.prototype.onclose(new CloseEvent('close', { code: 1006, reason: 'Connection failed' }));
      }
    });
  }

  async simulateReconnection() {
    // Simulate disconnection first
    await this.simulateConnectionFailure();

    // Then simulate reconnection after delay
    setTimeout(async () => {
      await this.page.evaluate(() => {
        const mockWS = (window as any).WebSocket;
        if (mockWS.prototype && mockWS.prototype.onopen) {
          mockWS.prototype.onopen(new Event('open'));
        }
      });
    }, 2000);
  }
}