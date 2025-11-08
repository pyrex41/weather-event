# Weather Event Flight Scheduling System

A flight scheduling application that automatically monitors weather conditions and cancels/reschedules flight lessons when weather becomes unsafe. Built with Rust (Axum), Elm, and SQLite.

## Features

- ✈️ **Automated Weather Monitoring**: Hourly checks of upcoming flights against weather conditions
- 🌦️ **Training Level-Specific Safety**: Different weather minimums for student pilots, private pilots, and instrument-rated pilots
- 🤖 **AI-Powered Rescheduling**: Uses OpenAI to suggest optimal alternative times
- 📱 **Real-time Notifications**: WebSocket push notifications, email, and SMS alerts
- 📊 **Full-Stack Dashboard**: Elm frontend with live updates

## Architecture

```
┌─────────────┐
│  Elm SPA    │ ← WebSocket + HTTP API
└─────────────┘
       ↓
┌─────────────┐
│ Axum Server │ → SQLite Database
└─────────────┘
       ↓
┌─────────────────────────────────┐
│  External APIs:                 │
│  - OpenWeatherMap (weather)     │
│  - OpenAI (AI rescheduling)     │
│  - Resend (email)               │
│  - Twilio (SMS, optional)       │
└─────────────────────────────────┘
```

## Tech Stack

- **Backend**: Rust 1.75+ with Axum web framework
- **Frontend**: Elm 0.19 with Vite build tooling
- **Database**: SQLite with sqlx for type-safe queries
- **Scheduler**: tokio-cron-scheduler for background jobs
- **Real-time**: WebSocket via Axum + tokio broadcast channels

## Prerequisites

- Rust 1.75 or higher
- Node.js 20+ and npm
- Elm 0.19
- SQLite
- API keys for:
  - OpenWeatherMap (required)
  - OpenAI (required)
  - Resend (required for email)
  - Twilio (optional for SMS)

## Local Development

### 1. Clone and Setup

```bash
# Clone repository
git clone <repo-url>
cd weather-event

# Copy environment template
cp .env.template .env

# Edit .env and add your API keys
nano .env
```

### 2. Database Setup

```bash
# Database will be created automatically on first run
# Migrations will be applied automatically
```

### 3. Run Backend

```bash
# Build and run the server
cargo run --release

# Server will start on http://localhost:3000
```

### 4. Run Frontend (Development)

```bash
cd elm
npm install
npm run dev

# Frontend dev server will start on http://localhost:5173
```

### 5. Build Frontend for Production

```bash
cd elm
npm run build

# Output will be in elm/dist/
# Copy to server's dist/ directory:
mkdir -p ../server/dist
cp -r dist/* ../server/dist/
```

## Environment Variables

Create a `.env` file in the project root:

```env
# Database
DATABASE_URL=sqlite:weather_app.db

# OpenWeatherMap API
WEATHER_API_KEY=your_key_here
WEATHER_API_BASE_URL=https://api.openweathermap.org/data/2.5

# OpenAI API
OPENAI_API_KEY=sk-proj-...

# Resend Email API
RESEND_API_KEY=re_...
FROM_EMAIL=alerts@flightschedulepro.com

# Twilio SMS (optional)
TWILIO_ACCOUNT_SID=AC...
TWILIO_AUTH_TOKEN=...
TWILIO_FROM_NUMBER=+1234567890

# Logging
RUST_LOG=info,server=debug
```

## API Documentation

### REST Endpoints

#### Health Check
```bash
GET /health
# Response: {"status": "ok"}
```

#### Bookings

```bash
# List all bookings
GET /api/bookings

# Get specific booking
GET /api/bookings/:id

# Create booking
POST /api/bookings
Content-Type: application/json

{
  "student_id": "uuid",
  "scheduled_date": "2024-01-15T14:00:00Z",
  "departure_location": {
    "lat": 33.8113,
    "lon": -118.1515,
    "name": "KTOA"
  }
}
```

#### Students

```bash
# List all students
GET /api/students

# Create student
POST /api/students
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com",
  "phone": "+1234567890",
  "training_level": "STUDENT_PILOT"
}
```

### WebSocket

```bash
# Connect to WebSocket
ws://localhost:3000/ws

# Notifications format:
{
  "type": "WEATHER_CONFLICT",
  "booking_id": "uuid",
  "message": "Flight cancelled: High winds",
  "student_name": "John Doe",
  "original_date": "2024-01-15T14:00:00Z"
}
```

## Testing

### Unit Tests

Run unit tests for the core library:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test weather::tests::test_unit_conversions
cargo test weather::safety::tests
cargo test ai::tests
```

### Integration Tests

The project includes comprehensive integration tests:

```bash
# Run all integration tests
cargo test --test '*'

# Run weather integration tests
cargo test --test weather_integration_test

# Run database integration tests
cargo test --test database_integration_test

# Run specific integration test
cargo test --test weather_integration_test test_student_pilot_weather_safety_integration
```

**Integration Test Coverage:**
- ✅ Weather safety logic for all training levels
- ✅ Training level progression (Student → Private → Instrument)
- ✅ Weather scoring consistency
- ✅ Edge cases (at minimums, below minimums, unlimited ceiling)
- ✅ Multiple violation handling
- ✅ Database CRUD operations (students, bookings)
- ✅ Foreign key constraints and cascade deletes
- ✅ Concurrent write safety (10 concurrent operations)
- ✅ JSON serialization for complex types
- ✅ Status transitions for bookings

### Frontend Testing

```bash
cd elm

# Install dependencies (if not already done)
npm install

# Run Elm in dev mode with hot reload
npm run dev

# Build for production
npm run build

# The built files will be in elm/dist/
```

### E2E Testing

The project includes comprehensive E2E tests using Playwright to ensure the full application works end-to-end:

```bash
# Install E2E test dependencies
cd e2e
npm install

# Install Playwright browsers
npx playwright install

# Run E2E tests locally
./run-tests.sh

# Or run directly
npm run test

# Run tests in headed mode (visible browser)
npm run test:headed

# Run tests with UI mode (interactive)
npm run test:ui

# Debug tests
npm run test:debug

# View test reports
npm run report
```

**E2E Test Coverage:**
- ✅ Booking creation flow (happy path + validation + loading states)
- ✅ Real-time weather alerts via WebSocket
- ✅ WebSocket connection status and reconnection
- ✅ AI-powered reschedule flow with availability badges
- ✅ Student management with dashboard stats
- ✅ Error scenarios (API failures, timeouts, validation)
- ✅ Multi-browser support (Chromium, Firefox, WebKit)

**Test Execution Time:** < 30 seconds with aggressive mocking

**CI Integration:** Tests run automatically on GitHub Actions for all PRs and pushes to main/develop branches.

### Manual Testing

1. Start the backend server:
   ```bash
   cargo run --release
   ```

2. In another terminal, start the Elm dev server:
   ```bash
   cd elm && npm run dev
   ```

3. Open http://localhost:5173 in your browser

4. Test the WebSocket connection:
   - Check the status indicator in the header (should show "● Live")
   - Create a booking and watch for real-time updates
   - Open browser console to see WebSocket messages

5. Test API endpoints:
   ```bash
   # Health check
   curl http://localhost:3000/health

   # List bookings
   curl http://localhost:3000/api/bookings

   # List students
   curl http://localhost:3000/api/students
   ```

## Project Structure

```
weather-event/
├── core/                    # Core business logic library
│   ├── src/
│   │   ├── models.rs       # Database models and enums
│   │   ├── weather/
│   │   │   ├── api.rs      # OpenWeatherMap client
│   │   │   └── safety.rs   # Weather safety logic
│   │   ├── ai/
│   │   │   └── reschedule.rs  # AI rescheduling
│   │   └── notifications/
│   │       ├── email.rs    # Resend email
│   │       └── sms.rs      # Twilio/Mock SMS
│   └── Cargo.toml
├── server/                  # Axum web server
│   ├── src/
│   │   ├── main.rs         # Server entry point
│   │   ├── routes/         # API route handlers
│   │   └── scheduler.rs    # Background weather monitor
│   └── Cargo.toml
├── elm/                     # Elm frontend SPA
│   ├── src/
│   │   ├── Main.elm        # Main application entry
│   │   ├── Types.elm       # Type definitions
│   │   ├── Api.elm         # HTTP API client
│   │   ├── WebSocket.elm   # WebSocket ports
│   │   ├── main.js         # JS entry with WebSocket
│   │   └── style.css       # Styles
│   ├── elm.json
│   ├── package.json
│   ├── vite.config.js
│   └── index.html
├── tests/                   # Integration tests
│   ├── weather_integration_test.rs
│   ├── database_integration_test.rs
│   └── Cargo.toml
├── migrations/              # SQL database migrations
│   └── 001_init.sql
├── .env.template            # Environment variables template
└── Cargo.toml              # Workspace configuration
```

## How It Works

### Weather Monitoring Flow

1. **Scheduler** runs every hour
2. Queries all bookings in next 48 hours with status `SCHEDULED`
3. For each booking:
   - Fetches student's training level
   - Gets current weather for departure location
   - Checks if weather meets safety minimums
   - If unsafe:
     - Updates booking status to `CANCELLED`
     - Creates reschedule event record
     - Sends WebSocket notification to dashboard
     - Sends email with AI-generated reschedule options
     - Sends SMS alert

### Training Level Weather Minimums

| Level | Min Visibility | Max Wind | Min Ceiling | IMC Allowed |
|-------|---------------|----------|-------------|-------------|
| Student Pilot | 5 SM | 12 kt | 3000 ft | No |
| Private Pilot | 3 SM | 20 kt | 1000 ft | No |
| Instrument Rated | 1 SM | 30 kt | None | Yes |

All levels prohibit: Thunderstorms, Icing conditions

## Development Roadmap

- [x] Core backend with Axum
- [x] Database schema and migrations
- [x] Weather API integration
- [x] Safety checking logic
- [x] WebSocket notifications
- [x] Scheduler for automated checks
- [x] Elm frontend UI with real-time updates
- [x] AI reschedule integration with caching
- [x] Email/SMS notification infrastructure
- [x] Unit and integration tests (15+ test cases)
- [x] E2E tests with Playwright (25+ scenarios, <30s execution)
- [ ] Frontend npm package installation fixes
- [ ] Deployment configuration (Fly.io)
- [ ] Demo video

## Deployment

See deployment instructions in `docs/DEPLOY.md` (coming soon)

## License

MIT

## Support

For issues and questions, please open a GitHub issue.
