# Product Requirements Document: Weather Cancellation & AI Rescheduling (Rust + Elm Web Application)

## Project Summary

* **Organization**: Flight Schedule Pro
* **Category**: AI Solution
* **Estimated Time**: 3–5 days
* **Overview**: This system will automatically detect potential weather conflicts for scheduled flight lessons and use AI to intelligently manage notifications and suggest optimized rescheduling options. It will monitor weather at all critical locations (takeoff, landing, and flight corridor). The architecture is web-first using Rust backend with Elm frontend, deployed to Fly.io, with a clear path to add Tauri desktop wrapper as a stretch goal.

## 1. Core Objectives

The primary goals are to automate, optimize, and track the entire weather-related flight disruption process:

* Automate weather monitoring and flight conflict detection.
* Notify affected students and instructors in real-time via WebSocket push notifications.
* Generate AI-powered rescheduling options that consider student training levels and availability.
* Track all booking, cancellation, and reschedule data for analysis.
* Display active flight and weather alerts in a central Elm-based web dashboard.

## 2. Technical & Learning Goals

This project focuses on building a modern, robust event-driven system using Rust's concurrency and safety for real-time processing, with Elm for a type-safe functional frontend.

* **Learning Objectives**:
  * Build an event-driven notification and scheduling system using Tokio async runtime and broadcast channels for real-time updates.
  * Integrate real-time weather data APIs via HTTP clients like reqwest.
  * Implement AI reasoning for decision-making using direct OpenAI API integration or Rust frameworks like rig.
  * Work with Rust + Elm for full-stack type safety.
  * Design normalized database schemas for flight scheduling using sqlx for type-safe queries.
  * Build WebSocket-based real-time communication between Elm frontend and Rust backend.
  * Deploy to Fly.io with persistent SQLite storage.

* **Technical Stack**:
  * **Frontend**: Elm 0.19+ (compiles to JavaScript for browser rendering; pure functional UI with strong type safety).
  * **Backend**: Rust with Axum web framework (Tokio-native, modern async HTTP), sqlx (compile-time checked SQL), tokio-cron-scheduler (background jobs).
  * **Database**: SQLite (via sqlx for type-safe queries) on Fly.io persistent volume. Local-first architecture suitable for single-server deployment.
  * **Real-time**: WebSockets using axum-typed-websockets or tokio-tungstenite for live dashboard updates.
  * **AI**: Direct OpenAI API calls via reqwest, or rig framework for LLM orchestration if structured workflows needed.
  * **Notifications**:
    * Email: Resend or AWS SES (transactional emails for conflict alerts)
    * In-app: WebSocket push notifications to connected clients
    * SMS: Twilio integration with mock fallback (trait-based abstraction)
  * **Deployment**: Fly.io (single VM with persistent volume, ~$5/month)
  * **Weather API**: OpenWeatherMap (free tier: 1000 calls/day sufficient for prototype).

## 3. Architecture: Web-First with Tauri Compatibility

### **Primary Target (Days 1-4): Web Application on Fly.io**

```
┌─────────────────────────────────────┐
│   Elm Frontend (static JS bundle)  │
│   - Dashboard UI                     │
│   - WebSocket client                 │
└────────────┬────────────────────────┘
             │ HTTP/WebSocket
┌────────────▼────────────────────────┐
│   Axum Web Server (Rust)            │
│   - REST API endpoints               │
│   - WebSocket handler                │
│   - Static file serving              │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│   Shared Core Library (Rust)        │
│   - Business logic                   │
│   - Weather API integration          │
│   - AI rescheduling                  │
│   - Safety checks                    │
│   - SQLite queries (sqlx)            │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│   SQLite Database                    │
│   - Students, bookings, weather logs │
│   - Stored on Fly.io persistent vol  │
└─────────────────────────────────────┘
```

**Architecture Characteristics:**
- Single Rust binary serves both static Elm assets and API endpoints
- WebSocket connections maintained via Tokio broadcast channels
- Background scheduler runs hourly weather checks via tokio-cron
- SQLite on persistent disk (fast, ACID, perfect for <10K bookings)
- Fly.io handles HTTPS, load balancing, and persistent storage

### **Stretch Goal (Day 5): Tauri Desktop Wrapper**

For optional cross-platform desktop builds (macOS, Windows, Linux):
- Same Elm frontend runs in Tauri's webview
- Same Rust core logic wrapped as Tauri commands (IPC instead of HTTP)
- Local SQLite database (no server needed)
- Tauri Events replace WebSocket for real-time updates

**Design Principles for Dual-Mode Compatibility:**
1. **Shared Core Library**: Business logic, weather API, AI, safety checks live in `weather-event-core` crate with no Axum dependencies.
2. **Transport-Agnostic Elm Ports**: Use generic `sendCommand`/`receiveResponse` ports that work with both HTTP and Tauri IPC.
3. **Trait-Based I/O**: Abstract notifications, database access behind traits so implementations can swap (WebSocket vs Tauri Events).
4. **Local-First SQLite**: Database design works identically in server and desktop modes.

**What Changes Between Web and Tauri Modes:**

| Aspect              | Web Mode (Primary)        | Tauri Mode (Stretch)         |
|---------------------|---------------------------|------------------------------|
| Frontend-Backend    | HTTP REST + WebSocket     | Tauri IPC commands           |
| Deployment          | Remote Fly.io server      | Bundled desktop application  |
| Database Location   | Server persistent volume  | User's local filesystem      |
| Real-time Updates   | WebSocket broadcast       | Tauri event system           |
| Background Jobs     | Server-side cron (always on) | In-app scheduler (battery concerns) |
| Distribution        | Single web URL            | Platform-specific installers |

This architecture ensures the core functionality is fully reusable if Tauri is added later, with minimal refactoring needed.

## 4. Success Criteria

The project will be considered a success when all the following criteria are met:

* ✅ Weather conflicts are automatically and accurately detected.
* ✅ Notifications are successfully sent to all affected students and instructors.
* ✅ AI suggests optimal rescheduling times (e.g., 3 valid options with reasoning).
* ✅ Database accurately updates bookings and logs all reschedule actions.
* ✅ Dashboard displays live weather alerts and current flight statuses.
* ✅ AI logic correctly considers the student's training level (e.g., applying stricter weather limits for a Student Pilot vs. an Instrument Rated pilot).
* ✅ WebSocket connections maintain real-time updates without page refresh.
* ✅ Dashboard automatically shows new notifications when flights are cancelled (no manual refresh).
* ✅ SMS functionality is implemented with trait-based abstraction (logs to console if no Twilio credentials provided).
* ✅ SQLite handles concurrent read/write operations without database corruption.
* ✅ Fly.io deployment succeeds with application cold start time <30 seconds.
* ✅ Background weather monitoring runs reliably every hour via tokio-cron.

## 5. Mock Data & Key Specifications

The system must handle data structured as follows, with the AI specifically using Training Level to apply appropriate Weather Minimums. Types are shown as Rust enums and structs, with JSON examples for API payloads.

### Rust Type Definitions

```rust
// Student training levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingLevel {
    StudentPilot,
    PrivatePilot,
    InstrumentRated,
}

// Booking status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookingStatus {
    Scheduled,
    Cancelled,
    Rescheduled,
    Completed,
}

// Core data models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: String,              // UUID v4
    pub name: String,
    pub email: String,
    pub phone: String,
    pub training_level: TrainingLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    pub id: String,
    pub student_id: String,
    pub scheduled_date: DateTime<Utc>,
    pub departure_location: Location,
    pub destination_location: Option<Location>,
    pub status: BookingStatus,
}
```

### Example JSON Data Structures

**Sample Booking (API Response):**
```json
{
  "id": "clx7a2b3c4d5e6f7g8h9i0j1",
  "studentId": "clx1a2b3c4d5e6f7g8h9i0j1",
  "scheduledDate": "2025-11-10T14:00:00Z",
  "departureLocation": {
    "lat": 33.8366,
    "lon": -118.3519,
    "name": "Torrance Airport (KTOA)"
  },
  "destinationLocation": null,
  "status": "SCHEDULED"
}
```

**Weather API Response (OpenWeatherMap):**
```json
{
  "coord": { "lon": -118.3519, "lat": 33.8366 },
  "weather": [{ "main": "Clouds", "description": "broken clouds" }],
  "main": { "temp": 288.15, "pressure": 1013, "humidity": 65 },
  "visibility": 4800,  // meters (convert to statute miles: × 0.000621371)
  "wind": { "speed": 6.2, "deg": 270 },  // m/s (convert to knots: × 1.94384)
  "clouds": { "all": 75 },  // Cloud coverage percentage
  "dt": 1699635600
}
```

**AI Reschedule Output (Structured Response):**
```json
{
  "options": [
    {
      "dateTime": "2025-11-11T10:00:00Z",
      "reason": "Clear skies forecast, winds 5kt from west, instructor available, optimal VFR conditions for student pilot",
      "weatherScore": 9.2,
      "instructorAvailable": true
    },
    {
      "dateTime": "2025-11-11T14:00:00Z",
      "reason": "Mostly clear with scattered clouds at 3500ft, winds 8kt, good training conditions",
      "weatherScore": 8.7,
      "instructorAvailable": true
    },
    {
      "dateTime": "2025-11-12T09:00:00Z",
      "reason": "High pressure system moving in, light winds, excellent visibility",
      "weatherScore": 9.5,
      "instructorAvailable": false
    }
  ]
}
```

### Weather Minimums Logic

The AI and safety logic must apply these rules based on student training level (implement as Rust match statements or configurable database table):

| Training Level       | Weather Minimums                                                                 |
|----------------------|----------------------------------------------------------------------------------|
| **Student Pilot**    | Clear skies (no clouds below 3000ft), visibility > 5 statute miles, winds < 10 knots. Cannot fly in IMC. |
| **Private Pilot**    | Visibility > 3 statute miles, ceiling > 1000ft AGL, winds < 20 knots. VFR only. |
| **Instrument Rated** | IMC (Instrument Meteorological Conditions) acceptable. No thunderstorms, no icing conditions, no severe turbulence. |

**Implementation Note:** Make weather minimums configurable in database for easy adjustment without code changes:

```sql
CREATE TABLE weather_minimums (
    id TEXT PRIMARY KEY,
    training_level TEXT NOT NULL UNIQUE,
    min_visibility_sm REAL NOT NULL,
    max_wind_speed_kt REAL NOT NULL,
    min_ceiling_ft REAL,
    allow_imc BOOLEAN NOT NULL DEFAULT 0,
    no_thunderstorms BOOLEAN NOT NULL DEFAULT 1,
    no_icing BOOLEAN NOT NULL DEFAULT 1
);
```

## 6. Testing Checklist

The following tests must pass to ensure stability and correctness:

* **Weather API Integration**: System returns valid JSON for each required location (use mock responses in tests with `wiremock` or similar).
* **Safety Logic**: System correctly flags unsafe conditions based on student training level and weather minimums (unit tests with property-based testing for edge cases).
* **AI Output**: AI successfully generates at least 3 valid reschedule options with structured output (integration test with mocked OpenAI responses).
* **Notification**:
  * Emails sent successfully upon conflict detection (verify with email sandbox/mock SMTP)
  * WebSocket broadcasts received by all connected clients
  * SMS mock logger confirms messages sent to correct phone numbers
* **Dashboard**: Elm UI displays live alerts and accurate flight statuses (end-to-end tests with Playwright or Selenium).
* **Database**:
  * Reschedules are logged and tracked correctly with full audit trail
  * Concurrent operations don't corrupt SQLite database
  * Foreign key constraints prevent orphaned records
* **Scheduler**: Background weather-monitoring process runs on schedule via tokio-cron (assert job execution in integration tests).
* **WebSocket**: Connections stay alive, automatically reconnect on disconnect, and handle backpressure correctly.

**Testing Stack:**
- Unit tests: `cargo test` (Rust standard testing)
- Integration tests: `wiremock` for API mocking, `sqlx::test` for database
- Property-based: `proptest` for safety logic fuzzing
- E2E tests: Playwright for browser automation

**Coverage Target:** 80%+ for core business logic (safety checks, AI prompts, database queries).

## 7. Deliverables & Metrics

### Required Deliverables

* **GitHub Repository**:
  * Clean code organized as Cargo workspace with `core` library and `server` binary
  * README with setup instructions: `cargo run` for dev, `fly deploy` for production
  * `.env.template` file listing all required environment variables
  * Database migrations in `migrations/` directory
  * Elm source code in `elm/` directory with build script
* **Demo Video (5–10 minutes)**:
  * Show flight creation in web dashboard
  * Simulate weather conflict detection (modify mock weather data)
  * Display AI-generated reschedule options
  * Demonstrate real-time WebSocket notifications
  * Show email notification received
  * Confirm reschedule and verify database update
  * Show mobile-responsive design (bonus)
* **Fly.io Deployment**:
  * Live web application accessible via public URL
  * Persistent SQLite database on Fly.io volume
  * HTTPS enabled via Fly.io proxy
  * Environment variables configured via `fly secrets`

### Key Metrics to Track

* **Bookings Created**: Total number of flight lessons scheduled
* **Weather Conflicts Detected**: Automatic cancellations due to unsafe weather
* **Successful Reschedules**: System-suggested options confirmed by students
* **Average Rescheduling Time**: From cancellation detection to new booking confirmation
* **API Costs**: Track OpenAI token usage and weather API call counts
* **WebSocket Connection Uptime**: Percentage of time real-time updates working

## 8. Deployment & Infrastructure

### **Platform: Fly.io** (Recommended)

**Why Fly.io:**
- ✅ Simple Rust deployment (`fly deploy` auto-detects Dockerfile)
- ✅ Persistent volumes for SQLite (survives restarts)
- ✅ Free tier: 3 shared-cpu VMs, 3GB persistent storage
- ✅ Global CDN and automatic HTTPS
- ✅ Environment variable management via CLI

**Infrastructure Components:**
- **Application**: Single Rust binary (Axum server + static Elm assets)
- **Database**: SQLite on 1GB persistent volume mounted at `/data`
- **Compute**: `shared-cpu-1x` VM with 256MB RAM (sufficient for prototype)
- **Networking**: HTTPS via Fly.io proxy, WebSocket support enabled

### Build Process

1. **Compile Elm Frontend:**
   ```bash
   cd elm && elm make src/Main.elm --output=../static/elm.js --optimize
   ```

2. **Build Rust Binary:**
   ```bash
   cargo build --release
   ```

3. **Deploy to Fly.io:**
   ```bash
   fly deploy
   ```

### Fly.io Configuration (`fly.toml`)

```toml
app = "weather-event-app"
primary_region = "lax"

[build]
  builder = "paketobuildpacks/builder:base"

[mounts]
  source = "weather_app_data"
  destination = "/data"

[[services]]
  internal_port = 3000
  protocol = "tcp"

  [[services.ports]]
    port = 80
    handlers = ["http"]

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  [[services.http_checks]]
    interval = "30s"
    timeout = "5s"
    method = "GET"
    path = "/health"
```

### Environment Variables

Create `.env` for local development and set via `fly secrets` for production:

```bash
# Database
DATABASE_URL=sqlite:///data/weather_app.db

# Weather API
WEATHER_API_KEY=your_openweathermap_key_here
WEATHER_API_BASE_URL=https://api.openweathermap.org/data/2.5

# OpenAI for AI rescheduling
OPENAI_API_KEY=sk-proj-...
OPENAI_MODEL=gpt-4o-mini

# Email notifications
RESEND_API_KEY=re_...
FROM_EMAIL=alerts@flightschedulepro.com

# SMS notifications (optional - leave empty for mock mode)
TWILIO_ACCOUNT_SID=
TWILIO_AUTH_TOKEN=
TWILIO_FROM_NUMBER=

# Application
RUST_LOG=info,weather_event=debug
PORT=3000
```

**Setting secrets in Fly.io:**
```bash
fly secrets set OPENAI_API_KEY=sk-proj-... WEATHER_API_KEY=... RESEND_API_KEY=...
```

### Cost Estimates

**Target: <$10/month for prototype**

| Service | Free Tier | Usage Strategy | Estimated Cost |
|---------|-----------|----------------|----------------|
| Fly.io | 3 shared VMs, 3GB storage | Use 1 VM + 1GB volume | $0 (within free tier) |
| OpenWeatherMap | 1000 calls/day | Check only flights in next 48hrs (~50 calls/day) | $0 |
| OpenAI API | Pay-per-use | gpt-4o-mini ($0.15/1M input, $0.60/1M output) ~1000 tokens/reschedule × 10 conflicts/day = $0.02/day | ~$0.60/month |
| Resend (Email) | 3000 emails/month | 2 emails per conflict (student + instructor) × 10/day = 600/month | $0 |
| Domain (optional) | N/A | Use free Fly.io subdomain initially | $0 |
| **Total** | | | **$0.60/month** |

**Cost Controls:**
- Cache weather data (1 hour TTL) to reduce API calls
- Rate limit AI requests (max 1 per booking per conflict)
- Only check flights scheduled in next 48 hours
- Use `gpt-4o-mini` instead of `gpt-4` (10x cheaper)

## 9. SMS Notifications with Twilio

**Requirement**: SMS functionality must be implemented, but work without Twilio credentials during development/demo.

### Trait-Based Abstraction

```rust
// src/notifications/sms.rs
use async_trait::async_trait;

#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, to: &str, message: &str) -> Result<(), Box<dyn std::error::Error>>;
}

// Real Twilio implementation
pub struct TwilioProvider {
    account_sid: String,
    auth_token: String,
    from_number: String,
    client: reqwest::Client,
}

#[async_trait]
impl SmsProvider for TwilioProvider {
    async fn send_sms(&self, to: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        let response = self.client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&[
                ("To", to),
                ("From", &self.from_number),
                ("Body", message),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Twilio API error: {}", response.status()).into())
        }
    }
}

// Mock implementation for development
pub struct MockSmsProvider;

#[async_trait]
impl SmsProvider for MockSmsProvider {
    async fn send_sms(&self, to: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("📱 [MOCK SMS] To: {}, Message: {}", to, message);
        Ok(())
    }
}

// Factory function - automatically chooses based on environment
pub fn create_sms_provider() -> Box<dyn SmsProvider> {
    if let (Ok(sid), Ok(token), Ok(from)) = (
        std::env::var("TWILIO_ACCOUNT_SID"),
        std::env::var("TWILIO_AUTH_TOKEN"),
        std::env::var("TWILIO_FROM_NUMBER"),
    ) {
        tracing::info!("✅ Using real Twilio SMS provider");
        Box::new(TwilioProvider {
            account_sid: sid,
            auth_token: token,
            from_number: from,
            client: reqwest::Client::new(),
        })
    } else {
        tracing::warn!("⚠️  Twilio credentials not found, using mock SMS provider");
        Box::new(MockSmsProvider)
    }
}
```

**Benefits:**
- ✅ Testable without real Twilio account
- ✅ Demo video shows SMS messages in logs
- ✅ Easy to enable real SMS later (just add credentials)
- ✅ Same code path for both mock and real implementations

## 10. Bonus Features (Optional)

These features are for consideration if time allows after core deliverables are met:

* **Tauri Desktop Application** (Stretch Goal - Day 5):
  * Cross-platform builds for macOS, Windows, Linux
  * Reuse Elm frontend and Rust core logic
  * Local SQLite database (no server needed)
  * Native system notifications
  * Installers for each platform

* **Real SMS Notifications**:
  * Obtain Twilio phone number and configure credentials
  * Already implemented via trait system - just add env vars

* **Google Calendar Integration**:
  * OAuth flow for student calendar access
  * Automatically create calendar events for bookings
  * Update events when rescheduled

* **Historical Weather Analytics**:
  * Store weather check history in database
  * Dashboard showing cancellation patterns
  * Identify "high risk" time periods for scheduling
  * Use `polars` or `arrow` for efficient data analysis

* **Mobile-Responsive Dashboard**:
  * Elm UI optimized for mobile browsers
  * Touch-friendly controls
  * PWA (Progressive Web App) support for "install to home screen"

* **Advanced AI Features**:
  * Predictive cancellation model (ML to forecast conflicts before they occur)
  * Learning from user preferences (which reschedule times are accepted most)
  * Multi-student batch rescheduling (if instructor cancels, reschedule all their students)

* **Enhanced Real-Time Features**:
  * Server-Sent Events fallback if WebSocket unavailable
  * Optimistic UI updates (instant feedback before server confirms)
  * Offline mode with sync when connection restored

## 11. File Structure

Recommended project organization:

```
weather-event/
├── Cargo.toml              # Workspace root
├── fly.toml                # Fly.io deployment config
├── .env.template           # Environment variable template
├── README.md               # Setup and deployment instructions
├── Dockerfile              # Container build (if not using buildpacks)
│
├── migrations/             # SQLite schema migrations
│   ├── 001_init.sql
│   ├── 002_weather_logs.sql
│   └── 003_weather_minimums.sql
│
├── core/                   # Shared business logic library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── models.rs       # Student, Booking, Weather structs
│       ├── db.rs           # Database connection and queries
│       ├── weather/
│       │   ├── mod.rs
│       │   ├── api.rs      # OpenWeatherMap integration
│       │   └── safety.rs   # Conflict detection logic
│       ├── ai/
│       │   └── reschedule.rs  # AI rescheduling logic
│       └── notifications/
│           ├── mod.rs
│           ├── email.rs    # Resend integration
│           └── sms.rs      # Twilio trait + mock
│
├── server/                 # Web server binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Axum server setup
│       ├── routes/
│       │   ├── mod.rs
│       │   ├── bookings.rs # CRUD endpoints
│       │   ├── students.rs
│       │   └── websocket.rs  # WebSocket handler
│       ├── scheduler.rs    # Background weather checks (tokio-cron)
│       └── state.rs        # Shared application state
│
├── elm/                    # Elm frontend
│   ├── elm.json
│   ├── package.json        # Build scripts
│   └── src/
│       ├── Main.elm        # Entry point
│       ├── Types.elm       # Type definitions
│       ├── Api.elm         # HTTP requests to Rust backend
│       ├── WebSocket.elm   # WebSocket ports
│       └── Pages/
│           ├── Dashboard.elm
│           ├── Bookings.elm
│           └── Alerts.elm
│
└── static/                 # Compiled frontend assets
    ├── index.html
    ├── elm.js              # Compiled from elm/
    ├── styles.css
    └── websocket.js        # JS glue for Elm ports
```

This structure maintains clear separation between:
- **`core/`**: Reusable business logic (importable by server or future Tauri app)
- **`server/`**: Web-specific code (Axum routes, HTTP handlers)
- **`elm/`**: Frontend source code
- **`static/`**: Compiled frontend ready to serve

## 12. AI Rescheduling Implementation Details

### Structured Output Approach

Use JSON schema or structured prompts to ensure AI returns parseable data:

```rust
// src/ai/reschedule.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct RescheduleOption {
    pub date_time: DateTime<Utc>,
    pub reason: String,
    pub weather_score: f32,
    pub instructor_available: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RescheduleResponse {
    pub options: Vec<RescheduleOption>,
}

pub async fn generate_reschedule_options(
    booking: &Booking,
    student: &Student,
    weather_forecast: &[WeatherData],
    instructor_schedule: &[Booking],
) -> Result<Vec<RescheduleOption>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Build context for AI
    let context = format!(
        r#"A flight lesson was cancelled due to weather.

Student: {} (Training Level: {:?})
Original Time: {}
Location: {}

Weather Forecast (next 7 days):
{}

Instructor Availability (booked times):
{}

Weather Minimums for {:?}:
{}

Task: Suggest exactly 3 optimal reschedule times in the next 7 days.
Consider weather conditions, instructor availability, and student training requirements.
Return ONLY valid JSON matching this schema:
{{
  "options": [
    {{
      "dateTime": "ISO8601 string",
      "reason": "explanation",
      "weatherScore": 0-10,
      "instructorAvailable": boolean
    }}
  ]
}}
"#,
        student.name,
        student.training_level,
        booking.scheduled_date,
        booking.departure_location.name,
        format_weather_forecast(weather_forecast),
        format_instructor_schedule(instructor_schedule),
        student.training_level,
        get_weather_minimums(&student.training_level),
    );

    let request = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": "You are a flight scheduling assistant. Always return valid JSON."
            },
            {
                "role": "user",
                "content": context
            }
        ],
        "temperature": 0.7,
        "response_format": { "type": "json_object" }
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", std::env::var("OPENAI_API_KEY")?))
        .json(&request)
        .send()
        .await?
        .json::<OpenAIResponse>()
        .await?;

    // Parse structured JSON response
    let reschedule_response: RescheduleResponse = serde_json::from_str(
        &response.choices[0].message.content
    )?;

    // Fallback: if AI fails to return 3 options, generate rule-based options
    if reschedule_response.options.len() < 3 {
        tracing::warn!("AI returned fewer than 3 options, using fallback logic");
        return Ok(generate_fallback_options(booking, student, weather_forecast));
    }

    Ok(reschedule_response.options)
}

// Fallback rule-based scheduling if AI fails
fn generate_fallback_options(
    booking: &Booking,
    student: &Student,
    weather_forecast: &[WeatherData],
) -> Vec<RescheduleOption> {
    // Simple logic: find next 3 days with good weather
    weather_forecast
        .iter()
        .filter(|w| is_weather_safe(&student.training_level, w))
        .take(3)
        .map(|w| RescheduleOption {
            date_time: w.date_time,
            reason: "Good weather conditions based on rule-based analysis".to_string(),
            weather_score: calculate_weather_score(&student.training_level, w),
            instructor_available: false, // Would need to check DB
        })
        .collect()
}
```

### Cost Optimization

```rust
// Cache AI responses to avoid redundant API calls
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct AiCache {
    cache: RwLock<HashMap<String, (RescheduleResponse, DateTime<Utc>)>>,
}

impl AiCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_or_generate(
        &self,
        cache_key: &str,
        ttl_hours: i64,
        generator: impl Future<Output = Result<RescheduleResponse, Box<dyn std::error::Error>>>,
    ) -> Result<RescheduleResponse, Box<dyn std::error::Error>> {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some((response, cached_at)) = cache.get(cache_key) {
                if Utc::now().signed_duration_since(*cached_at).num_hours() < ttl_hours {
                    tracing::info!("💰 Using cached AI response (saved API call)");
                    return Ok(response.clone());
                }
            }
        }

        // Generate new response
        let response = generator.await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key.to_string(), (response.clone(), Utc::now()));
        }

        Ok(response)
    }
}
```

**Usage:**
```rust
let cache_key = format!("reschedule_{}_{}", booking.id, booking.scheduled_date);
let options = ai_cache.get_or_generate(&cache_key, 6, async {
    generate_reschedule_options(&booking, &student, &forecast, &schedule).await
}).await?;
```

---

**This PRD provides a comprehensive, production-ready specification for a Rust + Elm weather cancellation and AI rescheduling system, deployable to Fly.io in 3-5 days, with a clear path to add Tauri desktop wrapper as a stretch goal.**
