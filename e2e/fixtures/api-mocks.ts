// API mocking utilities for Playwright tests
import { Page } from '@playwright/test';
import { GOOD_WEATHER, SEVERE_WEATHER, RESCHEDULE_OPTIONS } from './test-data';

export class ApiMocks {
  private page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async setupWeatherApiMocks() {
    // Mock OpenWeatherMap API
    await this.page.route('**/data/2.5/weather*', async (route) => {
      const url = new URL(route.request().url());
      const lat = url.searchParams.get('lat');
      const lon = url.searchParams.get('lon');

      // Simulate severe weather for certain coordinates
      if (lat === '41.9786' && lon === '-87.9048') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(SEVERE_WEATHER)
        });
      } else {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(GOOD_WEATHER)
        });
      }
    });
  }

  async setupOpenAIMocks() {
    // Mock OpenAI API for reschedule suggestions
    await this.page.route('**/v1/chat/completions', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          choices: [{
            message: {
              content: JSON.stringify(RESCHEDULE_OPTIONS)
            }
          }]
        })
      });
    });
  }

  async setupBackendApiMocks() {
    // Mock backend API endpoints
    await this.page.route('**/api/students', async (route) => {
      if (route.request().method() === 'GET') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify([
            {
              id: '1',
              name: 'John Doe',
              email: 'john.doe@example.com',
              training_level: 'PRIVATE_PILOT',
              phone: '+1-555-0123'
            }
          ])
        });
      } else if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 201,
          contentType: 'application/json',
          body: JSON.stringify({
            id: '2',
            name: 'Jane Smith',
            email: 'jane.smith@example.com',
            training_level: 'PRIVATE_PILOT',
            phone: '+1-555-0456'
          })
        });
      }
    });

    await this.page.route('**/api/bookings', async (route) => {
      if (route.request().method() === 'GET') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify([
            {
              id: '1',
              student_id: '1',
              aircraft_type: 'Cessna 172',
              scheduled_date: '2025-11-08T10:00:00Z',
              departure_location: {
                lat: 41.9786,
                lon: -87.9048,
                name: 'KORD'
              },
              status: 'Confirmed'
            }
          ])
        });
      } else if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 201,
          contentType: 'application/json',
          body: JSON.stringify({
            id: '2',
            student_id: '1',
            aircraft_type: 'Cessna 172',
            scheduled_date: '2025-11-08T14:00:00Z',
            departure_location: {
              lat: 41.9786,
              lon: -87.9048,
              name: 'KORD'
            },
            status: 'Confirmed'
          })
        });
      }
    });
  }

  async setupAllMocks() {
    await this.setupWeatherApiMocks();
    await this.setupOpenAIMocks();
    await this.setupBackendApiMocks();
  }
}