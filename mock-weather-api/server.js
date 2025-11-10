const express = require('express');
const cors = require('cors');

const app = express();
const PORT = 3001;

// Enable CORS for all routes
app.use(cors());

// Mock weather data for different locations
const mockWeatherData = {
  // Los Angeles (KTOA area)
  '33.8113,-118.1515': {
    lat: 33.8113,
    lon: -118.1515,
    timezone: "America/Los_Angeles",
    timezone_offset: -28800,
    current: {
      dt: Math.floor(Date.now() / 1000),
      sunrise: Math.floor(Date.now() / 1000) - 21600,
      sunset: Math.floor(Date.now() / 1000) + 21600,
      temp: 295.65, // 72.5°F in Kelvin
      feels_like: 298.35, // 75.2°F in Kelvin
      pressure: 1013,
      humidity: 65,
      dew_point: 289.15, // 60.8°F in Kelvin
      uvi: 6.2,
      clouds: 20,
      visibility: 10000,
      wind_speed: 8.2,
      wind_deg: 270,
      wind_gust: 12.1,
      weather: [
        {
          id: 801,
          main: "Clouds",
          description: "few clouds",
          icon: "02d"
        }
      ]
    },
    minutely: Array.from({length: 60}, (_, i) => ({
      dt: Math.floor(Date.now() / 1000) + (i * 60),
      precipitation: Math.random() * 0.1
    })),
    hourly: Array.from({length: 48}, (_, i) => ({
      dt: Math.floor(Date.now() / 1000) + (i * 3600),
      temp: 70 + Math.sin(i / 4) * 10 + Math.random() * 5,
      feels_like: 72 + Math.sin(i / 4) * 10 + Math.random() * 5,
      pressure: 1010 + Math.random() * 10,
      humidity: 60 + Math.random() * 20,
      dew_point: 55 + Math.random() * 10,
      uvi: Math.max(0, 8 - Math.abs(i - 12)),
      clouds: Math.floor(Math.random() * 50),
      visibility: 8000 + Math.random() * 2000,
      wind_speed: 5 + Math.random() * 10,
      wind_deg: 200 + Math.random() * 160,
      wind_gust: 8 + Math.random() * 15,
      weather: [
        {
          id: Math.random() > 0.7 ? 500 : 800,
          main: Math.random() > 0.7 ? "Rain" : "Clear",
          description: Math.random() > 0.7 ? "light rain" : "clear sky",
          icon: Math.random() > 0.7 ? "10d" : "01d"
        }
      ],
      pop: Math.random() * 0.3,
      rain: Math.random() > 0.8 ? { "1h": Math.random() * 2 } : undefined
    })),
    daily: Array.from({length: 8}, (_, i) => ({
      dt: Math.floor(Date.now() / 1000) + (i * 86400),
      sunrise: Math.floor(Date.now() / 1000) + (i * 86400) - 21600,
      sunset: Math.floor(Date.now() / 1000) + (i * 86400) + 21600,
      moonrise: Math.floor(Date.now() / 1000) + (i * 86400) - 18000,
      moonset: Math.floor(Date.now() / 1000) + (i * 86400) + 18000,
      moon_phase: Math.random(),
      summary: "Expect a day of partly cloudy with an average temperature of 75°F",
      temp: {
        day: 75 + Math.sin(i / 2) * 5 + Math.random() * 5,
        min: 65 + Math.sin(i / 2) * 3 + Math.random() * 3,
        max: 85 + Math.sin(i / 2) * 3 + Math.random() * 3,
        night: 70 + Math.sin(i / 2) * 4 + Math.random() * 4,
        eve: 78 + Math.sin(i / 2) * 4 + Math.random() * 4,
        morn: 68 + Math.sin(i / 2) * 4 + Math.random() * 4
      },
      feels_like: {
        day: 77 + Math.sin(i / 2) * 5 + Math.random() * 5,
        night: 72 + Math.sin(i / 2) * 4 + Math.random() * 4,
        eve: 80 + Math.sin(i / 2) * 4 + Math.random() * 4,
        morn: 70 + Math.sin(i / 2) * 4 + Math.random() * 4
      },
      pressure: 1012 + Math.random() * 8,
      humidity: 55 + Math.random() * 20,
      dew_point: 58 + Math.random() * 8,
      wind_speed: 6 + Math.random() * 8,
      wind_deg: 220 + Math.random() * 120,
      wind_gust: 10 + Math.random() * 12,
      weather: [
        {
          id: Math.random() > 0.6 ? 801 : 800,
          main: Math.random() > 0.6 ? "Clouds" : "Clear",
          description: Math.random() > 0.6 ? "few clouds" : "clear sky",
          icon: Math.random() > 0.6 ? "02d" : "01d"
        }
      ],
      clouds: Math.floor(Math.random() * 40),
      pop: Math.random() * 0.4,
      rain: Math.random() > 0.7 ? Math.random() * 5 : undefined,
      uvi: 6 + Math.random() * 4
    })),
    alerts: Math.random() > 0.8 ? [{
      sender_name: "National Weather Service",
      event: "Small Craft Advisory",
      start: Math.floor(Date.now() / 1000),
      end: Math.floor(Date.now() / 1000) + 86400,
      description: "Small Craft Advisory in effect until 6 PM PDT",
      tags: ["Marine"]
    }] : undefined
  },

  // Default fallback for any location
  'default': {
    lat: 40.7128,
    lon: -74.0060,
    timezone: "America/New_York",
    timezone_offset: -18000,
    current: {
      dt: Math.floor(Date.now() / 1000),
      sunrise: Math.floor(Date.now() / 1000) - 21600,
      sunset: Math.floor(Date.now() / 1000) + 21600,
      temp: 293.65, // 68.5°F in Kelvin
      feels_like: 294.35, // 70.2°F in Kelvin
      pressure: 1015,
      humidity: 70,
      dew_point: 287.65, // 58.8°F in Kelvin
      uvi: 4.2,
      clouds: 30,
      visibility: 9000,
      wind_speed: 6.2,
      wind_deg: 180,
      wind_gust: 9.1,
      weather: [
        {
          id: 802,
          main: "Clouds",
          description: "scattered clouds",
          icon: "03d"
        }
      ]
    },
    minutely: Array.from({length: 60}, (_, i) => ({
      dt: Math.floor(Date.now() / 1000) + (i * 60),
      precipitation: Math.random() * 0.05
    })),
    hourly: Array.from({length: 48}, (_, i) => ({
      dt: Math.floor(Date.now() / 1000) + (i * 3600),
      temp: 65 + Math.sin(i / 4) * 8 + Math.random() * 4,
      feels_like: 67 + Math.sin(i / 4) * 8 + Math.random() * 4,
      pressure: 1012 + Math.random() * 6,
      humidity: 65 + Math.random() * 15,
      dew_point: 52 + Math.random() * 8,
      uvi: Math.max(0, 6 - Math.abs(i - 12)),
      clouds: Math.floor(Math.random() * 60),
      visibility: 7000 + Math.random() * 3000,
      wind_speed: 4 + Math.random() * 8,
      wind_deg: 180 + Math.random() * 180,
      wind_gust: 7 + Math.random() * 10,
      weather: [
        {
          id: Math.random() > 0.6 ? 803 : 800,
          main: Math.random() > 0.6 ? "Clouds" : "Clear",
          description: Math.random() > 0.6 ? "broken clouds" : "clear sky",
          icon: Math.random() > 0.6 ? "04d" : "01d"
        }
      ],
      pop: Math.random() * 0.2,
      rain: Math.random() > 0.9 ? { "1h": Math.random() * 1 } : undefined
    })),
    daily: Array.from({length: 8}, (_, i) => ({
      dt: Math.floor(Date.now() / 1000) + (i * 86400),
      sunrise: Math.floor(Date.now() / 1000) + (i * 86400) - 21600,
      sunset: Math.floor(Date.now() / 1000) + (i * 86400) + 21600,
      moonrise: Math.floor(Date.now() / 1000) + (i * 86400) - 18000,
      moonset: Math.floor(Date.now() / 1000) + (i * 86400) + 18000,
      moon_phase: Math.random(),
      summary: "A mix of sun and clouds with mild temperatures",
      temp: {
        day: 70 + Math.sin(i / 2) * 4 + Math.random() * 4,
        min: 60 + Math.sin(i / 2) * 3 + Math.random() * 3,
        max: 80 + Math.sin(i / 2) * 3 + Math.random() * 3,
        night: 65 + Math.sin(i / 2) * 3 + Math.random() * 3,
        eve: 73 + Math.sin(i / 2) * 3 + Math.random() * 3,
        morn: 63 + Math.sin(i / 2) * 3 + Math.random() * 3
      },
      feels_like: {
        day: 72 + Math.sin(i / 2) * 4 + Math.random() * 4,
        night: 67 + Math.sin(i / 2) * 3 + Math.random() * 3,
        eve: 75 + Math.sin(i / 2) * 3 + Math.random() * 3,
        morn: 65 + Math.sin(i / 2) * 3 + Math.random() * 3
      },
      pressure: 1014 + Math.random() * 6,
      humidity: 60 + Math.random() * 15,
      dew_point: 55 + Math.random() * 6,
      wind_speed: 5 + Math.random() * 6,
      wind_deg: 200 + Math.random() * 160,
      wind_gust: 8 + Math.random() * 8,
      weather: [
        {
          id: Math.random() > 0.5 ? 802 : 800,
          main: Math.random() > 0.5 ? "Clouds" : "Clear",
          description: Math.random() > 0.5 ? "scattered clouds" : "clear sky",
          icon: Math.random() > 0.5 ? "03d" : "01d"
        }
      ],
      clouds: Math.floor(Math.random() * 50),
      pop: Math.random() * 0.3,
      rain: Math.random() > 0.8 ? Math.random() * 3 : undefined,
      uvi: 5 + Math.random() * 3
    }))
  }
};

// One Call API 3.0 endpoint
app.get('/data/3.0/onecall', (req, res) => {
  const { lat, lon } = req.query;

  if (!lat || !lon) {
    return res.status(400).json({ error: 'Missing lat/lon parameters' });
  }

  const key = `${lat},${lon}`;
  const data = mockWeatherData[key] || mockWeatherData['default'];

  // Add some randomness to make it feel real
  const responseData = JSON.parse(JSON.stringify(data));
  responseData.current.temp += (Math.random() - 0.5) * 4;
  responseData.current.wind_speed += (Math.random() - 0.5) * 2;

  console.log(`🌤️  Mock Weather API: Returning data for ${lat}, ${lon}`);
  res.json(responseData);
});

// Current weather endpoint (for backward compatibility)
app.get('/data/2.5/weather', (req, res) => {
  const { lat, lon } = req.query;

  if (!lat || !lon) {
    return res.status(400).json({ error: 'Missing lat/lon parameters' });
  }

  const key = `${lat},${lon}`;
  const data = mockWeatherData[key] || mockWeatherData['default'];

  const response = {
    coord: { lon: parseFloat(lon), lat: parseFloat(lat) },
    weather: data.current.weather,
    main: {
      temp: data.current.temp,
      feels_like: data.current.feels_like,
      pressure: data.current.pressure,
      humidity: data.current.humidity
    },
    visibility: data.current.visibility,
    wind: {
      speed: data.current.wind_speed,
      deg: data.current.wind_deg,
      gust: data.current.wind_gust
    },
    clouds: { all: data.current.clouds },
    dt: data.current.dt,
    name: "Mock Location"
  };

  console.log(`🌤️  Mock Current Weather API: Returning data for ${lat}, ${lon}`);
  res.json(response);
});

// Forecast endpoint (for backward compatibility)
app.get('/data/2.5/forecast', (req, res) => {
  const { lat, lon } = req.query;

  if (!lat || !lon) {
    return res.status(400).json({ error: 'Missing lat/lon parameters' });
  }

  const key = `${lat},${lon}`;
  const data = mockWeatherData[key] || mockWeatherData['default'];

  const response = {
    cod: "200",
    message: 0,
    cnt: 40,
    list: data.hourly.slice(0, 40).map(hour => ({
      dt: hour.dt,
      main: {
        temp: hour.temp,
        feels_like: hour.feels_like,
        pressure: hour.pressure,
        humidity: hour.humidity,
        temp_min: hour.temp - 2,
        temp_max: hour.temp + 2
      },
      weather: hour.weather,
      clouds: { all: hour.clouds },
      wind: {
        speed: hour.wind_speed,
        deg: hour.wind_deg,
        gust: hour.wind_gust
      },
      visibility: hour.visibility,
      pop: hour.pop,
      dt_txt: new Date(hour.dt * 1000).toISOString().replace('T', ' ').slice(0, -5)
    })),
    city: {
      id: 2643743,
      name: "Mock City",
      coord: { lat: parseFloat(lat), lon: parseFloat(lon) },
      country: "US",
      population: 100000,
      timezone: -18000,
      sunrise: data.current.sunrise,
      sunset: data.current.sunset
    }
  };

  console.log(`🌤️  Mock Forecast API: Returning data for ${lat}, ${lon}`);
  res.json(response);
});

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'Mock OpenWeatherMap API is running', timestamp: new Date().toISOString() });
});

app.listen(PORT, () => {
  console.log(`🌤️  Mock OpenWeatherMap API server running on http://localhost:${PORT}`);
  console.log(`📡 One Call API 3.0: http://localhost:${PORT}/data/3.0/onecall`);
  console.log(`📡 Current Weather: http://localhost:${PORT}/data/2.5/weather`);
  console.log(`📡 Forecast: http://localhost:${PORT}/data/2.5/forecast`);
  console.log(`💚 Health check: http://localhost:${PORT}/health`);
});