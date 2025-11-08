import { test, expect } from '@playwright/test';
import { ApiMocks } from '../fixtures/api-mocks';
import { WebSocketMock } from '../fixtures/websocket-mock';
import { TEST_STUDENT, TEST_BOOKING } from '../fixtures/test-data';
import {
  waitForLoadingToComplete,
  fillStudentForm,
  fillBookingForm,
  assertFormValidationError
} from '../utils/test-helpers';

test.describe('Booking Creation Flow', () => {
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

  test('should create student and booking successfully', async ({ page }) => {
    // Navigate to students page
    await page.click('[data-testid="nav-students"]');

    // Click create student button
    await page.click('[data-testid="create-student-btn"]');

    // Fill student form
    await fillStudentForm(page, TEST_STUDENT);

    // Submit form
    await page.click('[data-testid="submit-student-btn"]');

    // Verify success message
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="success-message"]')).toContainText('Student created successfully');

    // Navigate to bookings page
    await page.click('[data-testid="nav-bookings"]');

    // Click create booking button
    await page.click('[data-testid="create-booking-btn"]');

    // Fill booking form
    await fillBookingForm(page, TEST_BOOKING);

    // Submit form
    await page.click('[data-testid="submit-booking-btn"]');

    // Verify success message
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="success-message"]')).toContainText('Booking created successfully');

    // Verify booking appears in list
    await expect(page.locator('[data-testid="booking-list"]')).toContainText(TEST_BOOKING.aircraft_type);
    await expect(page.locator('[data-testid="booking-list"]')).toContainText(TEST_BOOKING.location);
  });

  test('should show validation errors for invalid student data', async ({ page }) => {
    // Navigate to students page
    await page.click('[data-testid="nav-students"]');
    await page.click('[data-testid="create-student-btn"]');

    // Try to submit empty form
    await page.click('[data-testid="submit-student-btn"]');

    // Check validation errors
    await assertFormValidationError(page, 'name', 'Name is required');
    await assertFormValidationError(page, 'email', 'Email is required');
    await assertFormValidationError(page, 'training-level', 'Training level is required');

    // Fill invalid email
    await page.fill('[data-testid="student-name"]', 'Test Student');
    await page.fill('[data-testid="student-email"]', 'invalid-email');
    await page.selectOption('[data-testid="student-training-level"]', 'Private Pilot');

    await page.click('[data-testid="submit-student-btn"]');

    // Check email validation
    await assertFormValidationError(page, 'email', 'Please enter a valid email address');
  });

  test('should show validation errors for invalid booking data', async ({ page }) => {
    // Navigate to bookings page
    await page.click('[data-testid="nav-bookings"]');
    await page.click('[data-testid="create-booking-btn"]');

    // Try to submit empty form
    await page.click('[data-testid="submit-booking-btn"]');

    // Check validation errors
    await assertFormValidationError(page, 'aircraft-type', 'Aircraft type is required');
    await assertFormValidationError(page, 'start-time', 'Start time is required');
    await assertFormValidationError(page, 'end-time', 'End time is required');
    await assertFormValidationError(page, 'location', 'Location is required');

    // Fill invalid time range (end before start)
    await page.selectOption('[data-testid="booking-aircraft"]', 'Cessna 172');
    await page.fill('[data-testid="booking-start-time"]', '2025-11-08T15:00:00');
    await page.fill('[data-testid="booking-end-time"]', '2025-11-08T14:00:00');
    await page.fill('[data-testid="booking-location"]', 'KORD');

    await page.click('[data-testid="submit-booking-btn"]');

    // Check time validation
    await assertFormValidationError(page, 'end-time', 'End time must be after start time');
  });

  test('should show loading states during form submission', async ({ page }) => {
    // Navigate to students page
    await page.click('[data-testid="nav-students"]');
    await page.click('[data-testid="create-student-btn"]');

    // Fill form
    await fillStudentForm(page, TEST_STUDENT);

    // Submit and check loading state
    await page.click('[data-testid="submit-student-btn"]');

    // Verify loading indicator appears
    await expect(page.locator('[data-testid="loading-spinner"]')).toBeVisible();

    // Verify submit button is disabled
    await expect(page.locator('[data-testid="submit-student-btn"]')).toBeDisabled();

    // Wait for completion
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();

    // Verify loading state is gone
    await expect(page.locator('[data-testid="loading-spinner"]')).not.toBeVisible();
    await expect(page.locator('[data-testid="submit-student-btn"]')).not.toBeDisabled();
  });
});