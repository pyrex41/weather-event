# Mock OpenWeatherMap API Server

A simple Node.js mock server that mimics OpenWeatherMap API responses for development and testing.

## Quick Start

```bash
cd mock-weather-api
npm install
npm start
```

The server will run on `http://localhost:3001` and provide mock responses for:

- **One Call API 3.0**: `/data/3.0/onecall`
- **Current Weather**: `/data/2.5/weather`
- **Forecast**: `/data/2.5/forecast`
- **Health Check**: `/health`

## Configuration

The server provides realistic weather data for:
- **Los Angeles (KTOA)**: `lat=33.8113, lon=-118.1515`
- **Any other location**: Returns default NYC-like data

Weather data includes:
- Current conditions with realistic temperature/wind
- 48-hour hourly forecast
- 8-day daily forecast
- Random weather alerts (20% chance)
- Proper aviation units (knots, miles, feet)

## Integration

Update your `.env` file to use the mock server:

```env
WEATHER_API_KEY=mock_weather_key
WEATHER_API_BASE_URL=http://localhost:3001/data/2.5
```

## Production Switch

When ready for production, update `.env` with real OpenWeatherMap credentials:

```env
WEATHER_API_KEY=your_real_api_key_here
WEATHER_API_BASE_URL=https://api.openweathermap.org/data/2.5
```

The One Call API 3.0 provides 1,000 free calls/day, then $0.0015 per call.