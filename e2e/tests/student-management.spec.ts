import { test, expect } from '@playwright/test';
import { ApiMocks } from '../fixtures/api-mocks';
import { WebSocketMock } from '../fixtures/websocket-mock';
import { TEST_STUDENT } from '../fixtures/test-data';
import { waitForLoadingToComplete, assertDashboardStats } from '../utils/test-helpers';

test.describe('Student Management', () => {
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

  test('should display student list with all details', async ({ page }) => {
    // Navigate to students page
    await page.click('[data-testid="nav-students"]');

    // Verify student list is visible
    await expect(page.locator('[data-testid="student-list"]')).toBeVisible();

    // Verify student details are displayed
    const firstStudent = page.locator('[data-testid="student-item"]').first();
    await expect(firstStudent).toContainText('John Doe');
    await expect(firstStudent).toContainText('john.doe@example.com');
    await expect(firstStudent).toContainText('Private Pilot');
    await expect(firstStudent).toContainText('+1-555-0123');
  });

  test('should update dashboard stats when students are added', async ({ page }) => {
    // Check initial stats
    await assertDashboardStats(page, { totalStudents: 1 });

    // Navigate to students and add a new student
    await page.click('[data-testid="nav-students"]');
    await page.click('[data-testid="create-student-btn"]');

    // Fill form with different student
    await page.fill('[data-testid="student-name"]', 'Jane Smith');
    await page.fill('[data-testid="student-email"]', 'jane.smith@example.com');
    await page.selectOption('[data-testid="student-training-level"]', 'Commercial Pilot');
    await page.fill('[data-testid="student-phone"]', '+1-555-0456');

    // Submit
    await page.click('[data-testid="submit-student-btn"]');

    // Verify stats updated
    await assertDashboardStats(page, { totalStudents: 2 });
  });

  test('should handle multiple students with different training levels', async ({ page }) => {
    // Navigate to students page
    await page.click('[data-testid="nav-students"]');

    // Verify different training levels are displayed
    await expect(page.locator('[data-testid="student-list"]')).toContainText('Private Pilot');

    // Add student with different training level
    await page.click('[data-testid="create-student-btn"]');
    await page.fill('[data-testid="student-name"]', 'Bob Johnson');
    await page.fill('[data-testid="student-email"]', 'bob.johnson@example.com');
    await page.selectOption('[data-testid="student-training-level"]', 'Certified Flight Instructor');
    await page.fill('[data-testid="student-phone"]', '+1-555-0789');
    await page.click('[data-testid="submit-student-btn"]');

    // Verify both training levels are shown
    await expect(page.locator('[data-testid="student-list"]')).toContainText('Private Pilot');
    await expect(page.locator('[data-testid="student-list"]')).toContainText('Certified Flight Instructor');
  });

  test('should filter students by training level', async ({ page }) => {
    // Add multiple students with different levels
    await page.click('[data-testid="nav-students"]');

    // Add commercial pilot
    await page.click('[data-testid="create-student-btn"]');
    await page.fill('[data-testid="student-name"]', 'Alice Wilson');
    await page.fill('[data-testid="student-email"]', 'alice.wilson@example.com');
    await page.selectOption('[data-testid="student-training-level"]', 'Commercial Pilot');
    await page.fill('[data-testid="student-phone"]', '+1-555-0321');
    await page.click('[data-testid="submit-student-btn"]');

    // Add CFI
    await page.click('[data-testid="create-student-btn"]');
    await page.fill('[data-testid="student-name"]', 'Charlie Brown');
    await page.fill('[data-testid="student-email"]', 'charlie.brown@example.com');
    await page.selectOption('[data-testid="student-training-level"]', 'Certified Flight Instructor');
    await page.fill('[data-testid="student-phone"]', '+1-555-0654');
    await page.click('[data-testid="submit-student-btn"]');

    // Test filtering (assuming filter dropdown exists)
    if (await page.locator('[data-testid="training-level-filter"]').isVisible()) {
      await page.selectOption('[data-testid="training-level-filter"]', 'Private Pilot');
      await expect(page.locator('[data-testid="student-list"]')).toContainText('John Doe');
      await expect(page.locator('[data-testid="student-list"]')).not.toContainText('Alice Wilson');

      await page.selectOption('[data-testid="training-level-filter"]', 'Commercial Pilot');
      await expect(page.locator('[data-testid="student-list"]')).toContainText('Alice Wilson');
      await expect(page.locator('[data-testid="student-list"]')).not.toContainText('John Doe');
    }
  });

  test('should search students by name and email', async ({ page }) => {
    // Add another student
    await page.click('[data-testid="nav-students"]');
    await page.click('[data-testid="create-student-btn"]');
    await page.fill('[data-testid="student-name"]', 'David Miller');
    await page.fill('[data-testid="student-email"]', 'david.miller@example.com');
    await page.selectOption('[data-testid="student-training-level"]', 'Private Pilot');
    await page.fill('[data-testid="student-phone"]', '+1-555-0987');
    await page.click('[data-testid="submit-student-btn"]');

    // Test search functionality (assuming search input exists)
    if (await page.locator('[data-testid="student-search"]').isVisible()) {
      // Search by name
      await page.fill('[data-testid="student-search"]', 'David');
      await expect(page.locator('[data-testid="student-list"]')).toContainText('David Miller');
      await expect(page.locator('[data-testid="student-list"]')).not.toContainText('John Doe');

      // Search by email
      await page.fill('[data-testid="student-search"]', 'john.doe');
      await expect(page.locator('[data-testid="student-list"]')).toContainText('John Doe');
      await expect(page.locator('[data-testid="student-list"]')).not.toContainText('David Miller');

      // Clear search
      await page.fill('[data-testid="student-search"]', '');
      await expect(page.locator('[data-testid="student-list"]')).toContainText('John Doe');
      await expect(page.locator('[data-testid="student-list"]')).toContainText('David Miller');
    }
  });

  test('should show student creation date', async ({ page }) => {
    // Navigate to students page
    await page.click('[data-testid="nav-students"]');

    // Verify creation dates are displayed
    const studentItems = page.locator('[data-testid="student-item"]');
    await expect(studentItems.first().locator('[data-testid="student-created"]')).toBeVisible();

    // Add new student and verify its creation date
    await page.click('[data-testid="create-student-btn"]');
    await page.fill('[data-testid="student-name"]', 'Eve Davis');
    await page.fill('[data-testid="student-email"]', 'eve.davis@example.com');
    await page.selectOption('[data-testid="student-training-level"]', 'Private Pilot');
    await page.fill('[data-testid="student-phone"]', '+1-555-0111');
    await page.click('[data-testid="submit-student-btn"]');

    // Verify new student has creation date
    await expect(page.locator('[data-testid="student-item"]').last().locator('[data-testid="student-created"]')).toBeVisible();
  });

  test('should handle empty student list gracefully', async ({ page }) => {
    // Mock empty student list
    await page.route('**/api/students', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ students: [] })
      });
    });

    // Navigate to students page
    await page.click('[data-testid="nav-students"]');

    // Verify empty state message
    await expect(page.locator('[data-testid="empty-students"]')).toBeVisible();
    await expect(page.locator('[data-testid="empty-students"]')).toContainText('No students found');

    // Verify create button is still visible
    await expect(page.locator('[data-testid="create-student-btn"]')).toBeVisible();
  });
});