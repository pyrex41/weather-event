// Test utility functions for E2E tests
import { Page, expect } from '@playwright/test';

export async function waitForLoadingToComplete(page: Page) {
  await page.waitForFunction(() => {
    const loaders = document.querySelectorAll('[data-testid="loading"], .loading, .spinner');
    return loaders.length === 0;
  }, { timeout: 10000 });
}

export async function waitForWebSocketConnection(page: Page) {
  await page.waitForFunction(() => {
    // Check for connection status indicator
    const statusElement = document.querySelector('[data-testid="ws-status"]');
    return statusElement && statusElement.textContent?.includes('Connected');
  }, { timeout: 5000 });
}

export async function fillStudentForm(page: Page, studentData: {
  name: string;
  email: string;
  training_level: string;
  phone: string;
}) {
  await page.fill('[data-testid="student-name"]', studentData.name);
  await page.fill('[data-testid="student-email"]', studentData.email);
  await page.selectOption('[data-testid="student-training-level"]', studentData.training_level);
  await page.fill('[data-testid="student-phone"]', studentData.phone);
}

export async function fillBookingForm(page: Page, bookingData: {
  aircraft_type: string;
  start_time: string;
  end_time: string;
  location: string;
  notes?: string;
}) {
  await page.selectOption('[data-testid="booking-aircraft"]', bookingData.aircraft_type);
  await page.fill('[data-testid="booking-start-time"]', bookingData.start_time);
  await page.fill('[data-testid="booking-end-time"]', bookingData.end_time);
  await page.fill('[data-testid="booking-location"]', bookingData.location);
  if (bookingData.notes) {
    await page.fill('[data-testid="booking-notes"]', bookingData.notes);
  }
}

export async function assertWeatherAlertVisible(page: Page, expectedText: string) {
  const alertElement = page.locator('[data-testid="weather-alert"]');
  await expect(alertElement).toBeVisible();
  await expect(alertElement).toContainText(expectedText);
}

export async function assertDashboardStats(page: Page, expectedStats: {
  totalStudents?: number;
  totalBookings?: number;
  activeAlerts?: number;
}) {
  if (expectedStats.totalStudents !== undefined) {
    await expect(page.locator('[data-testid="stat-students"]')).toContainText(expectedStats.totalStudents.toString());
  }
  if (expectedStats.totalBookings !== undefined) {
    await expect(page.locator('[data-testid="stat-bookings"]')).toContainText(expectedStats.totalBookings.toString());
  }
  if (expectedStats.activeAlerts !== undefined) {
    await expect(page.locator('[data-testid="stat-alerts"]')).toContainText(expectedStats.activeAlerts.toString());
  }
}

export async function assertFormValidationError(page: Page, fieldName: string, errorMessage: string) {
  const errorElement = page.locator(`[data-testid="error-${fieldName}"]`);
  await expect(errorElement).toBeVisible();
  await expect(errorElement).toContainText(errorMessage);
}

export async function assertWebSocketStatus(page: Page, expectedStatus: 'connecting' | 'connected' | 'disconnected') {
  const statusElement = page.locator('[data-testid="ws-status"]');
  await expect(statusElement).toHaveAttribute('data-status', expectedStatus);
}