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

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test weather::tests::test_unit_conversions
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
├── elm/                     # Elm frontend (to be built)
│   ├── src/
│   ├── package.json
│   └── vite.config.js
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
- [ ] Elm frontend UI
- [ ] AI reschedule integration
- [ ] Email/SMS notifications
- [ ] Unit and integration tests
- [ ] Deployment configuration
- [ ] Demo video

## Deployment

See deployment instructions in `docs/DEPLOY.md` (coming soon)

## License

MIT

## Support

For issues and questions, please open a GitHub issue.
