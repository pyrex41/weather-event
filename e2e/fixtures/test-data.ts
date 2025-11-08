// Test data fixtures for E2E tests

export const TEST_STUDENT = {
  name: 'John Doe',
  email: 'john.doe@example.com',
  training_level: 'Private Pilot',
  phone: '+1-555-0123'
};

export const TEST_BOOKING = {
  student_id: 1,
  instructor_id: 1,
  aircraft_type: 'Cessna 172',
  start_time: '2025-11-08T10:00:00Z',
  end_time: '2025-11-08T12:00:00Z',
  location: 'KORD',
  notes: 'Test booking for E2E'
};

export const GOOD_WEATHER = {
  coord: { lon: -87.9048, lat: 41.9786 },
  weather: [
    {
      id: 800,
      main: 'Clear',
      description: 'clear sky',
      icon: '01d'
    }
  ],
  base: 'stations',
  main: {
    temp: 15.0,
    feels_like: 14.5,
    temp_min: 12.0,
    temp_max: 18.0,
    pressure: 1013,
    humidity: 65
  },
  visibility: 10000,
  wind: {
    speed: 3.5,
    deg: 180
  },
  clouds: { all: 20 },
  dt: 1638360000,
  sys: {
    type: 2,
    id: 2011048,
    country: 'US',
    sunrise: 1638330000,
    sunset: 1638360000
  },
  timezone: -21600,
  id: 4887398,
  name: 'Chicago',
  cod: 200
};

export const SEVERE_WEATHER = {
  ...GOOD_WEATHER,
  weather: [
    {
      id: 200,
      main: 'Thunderstorm',
      description: 'thunderstorm with heavy rain',
      icon: '11d'
    }
  ],
  main: {
    ...GOOD_WEATHER.main,
    temp: 5.0,
    humidity: 95
  },
  wind: {
    speed: 15.0,
    deg: 270
  }
};

export const RESCHEDULE_OPTIONS = [
  {
    id: 1,
    start_time: '2025-11-08T14:00:00Z',
    end_time: '2025-11-08T16:00:00Z',
    instructor_available: true,
    weather_suitable: true,
    reason: 'Weather delay reschedule'
  },
  {
    id: 2,
    start_time: '2025-11-09T09:00:00Z',
    end_time: '2025-11-09T11:00:00Z',
    instructor_available: true,
    weather_suitable: true,
    reason: 'Alternative time slot'
  },
  {
    id: 3,
    start_time: '2025-11-09T13:00:00Z',
    end_time: '2025-11-09T15:00:00Z',
    instructor_available: false,
    weather_suitable: true,
    reason: 'Instructor unavailable'
  }
];