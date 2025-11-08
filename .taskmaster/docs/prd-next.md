# Product Requirements Document: Missing Features
**Flight Schedule Pro - Phase 2 Implementation**

## Document Control
- **Version**: 1.0
- **Date**: 2025-11-08
- **Status**: Draft
- **Owner**: Engineering Team

---

## Executive Summary

This PRD outlines the missing features identified through end-to-end test analysis. These features are essential for delivering a complete flight scheduling system with real-time weather integration, intelligent rescheduling, and robust error handling.

**Priority**: High
**Target Release**: Q1 2025
**Estimated Effort**: 8-12 weeks

---

## Table of Contents

1. [Feature 1: AI-Powered Reschedule System](#feature-1-ai-powered-reschedule-system)
2. [Feature 2: Real-Time Weather Alert System](#feature-2-real-time-weather-alert-system)
3. [Feature 3: Enhanced WebSocket Infrastructure](#feature-3-enhanced-websocket-infrastructure)
4. [Feature 4: Comprehensive Error Handling](#feature-4-comprehensive-error-handling)
5. [Feature 5: Loading States & User Feedback](#feature-5-loading-states--user-feedback)
6. [Feature 6: Form Validation Enhancements](#feature-6-form-validation-enhancements)
7. [Non-Functional Requirements](#non-functional-requirements)
8. [Success Metrics](#success-metrics)
9. [Implementation Roadmap](#implementation-roadmap)

---

## Feature 1: AI-Powered Reschedule System

### Overview
Provide flight instructors and students with intelligent, AI-generated reschedule options when weather or other factors require booking changes.

### User Stories

**US-1.1**: As a flight instructor, I want to click a "Reschedule" button on any booking so I can quickly find alternative time slots.

**US-1.2**: As a user, I want to see 3 AI-suggested reschedule options with clear reasoning so I can make an informed decision.

**US-1.3**: As a user, I want to see instructor availability badges on each option so I know which slots are actually available.

**US-1.4**: As a user, I want to see weather suitability indicators on each option so I can avoid rescheduling into bad weather.

**US-1.5**: As a user, I want to confirm or cancel a reschedule operation so I don't accidentally change bookings.

### Functional Requirements

#### FR-1.1: Reschedule Modal UI
- **Component**: RescheduleModal.elm
- **Trigger**: Clicking `[data-testid="reschedule-btn"]` on any booking card
- **Display Elements**:
  - Modal overlay with title "Reschedule Flight"
  - Original booking details (current time, location, aircraft)
  - 3 reschedule options displayed as cards
  - Close/Cancel button

#### FR-1.2: Reschedule Options Display
Each reschedule option must show:
- **Time Slot**: `[data-testid="option-time"]` - New proposed date/time
- **AI Reasoning**: `[data-testid="option-reason"]` - Why this option was suggested
- **Availability Badge**: `[data-testid="availability-badge"]`
  - "Available" (green) - Instructor is free
  - "Unavailable" (red/gray) - Instructor is booked
- **Weather Indicator**: `[data-testid="weather-indicator"]`
  - "Weather OK" (green) - Suitable flying conditions
  - "Marginal" (yellow) - Borderline conditions
  - "Not Suitable" (red) - Poor weather
- **Select Button**: `[data-testid="select-option-btn"]`
  - Enabled for available slots
  - Disabled for unavailable slots

#### FR-1.3: OpenAI Integration for Smart Suggestions
- **API**: POST `/api/bookings/{id}/reschedule-suggestions`
- **Request**:
  ```json
  {
    "booking_id": "string",
    "reason": "weather" | "instructor_unavailable" | "student_request",
    "constraints": {
      "preferred_times": ["morning", "afternoon", "evening"],
      "max_days_out": 7
    }
  }
  ```
- **Response**: 3 reschedule options with AI reasoning
- **OpenAI Prompt Template**:
  ```
  You are a flight scheduling assistant. The current booking:
  - Date/Time: {current_datetime}
  - Student Level: {training_level}
  - Aircraft: {aircraft_type}
  - Location: {airport_code}

  Reason for reschedule: {reason}
  Weather forecast: {forecast_data}
  Instructor availability: {availability_data}

  Suggest 3 alternative time slots with reasoning for each.
  ```

#### FR-1.4: Confirmation Dialog
- **Component**: ConfirmRescheduleModal.elm
- **Trigger**: Clicking `[data-testid="select-option-btn"]`
- **Display**:
  - `[data-testid="confirm-reschedule-modal"]`
  - Summary of changes (old time → new time)
  - "Confirm" button: `[data-testid="confirm-reschedule-btn"]`
  - "Cancel" button: `[data-testid="cancel-reschedule-btn"]`

#### FR-1.5: Backend API Endpoint
- **Endpoint**: PATCH `/api/bookings/{id}/reschedule`
- **Request**:
  ```json
  {
    "new_scheduled_date": "2025-11-09T14:00:00Z",
    "reason": "weather",
    "ai_suggestion_id": "optional-tracking-id"
  }
  ```
- **Response**: Updated booking object
- **Business Logic**:
  - Validate instructor availability
  - Check weather suitability
  - Update booking record
  - Send notifications (email, WebSocket)
  - Log reschedule history

#### FR-1.6: Loading and Error States
- Loading indicator: `[data-testid="reschedule-loading"]` during API calls
- Disable confirm button during processing
- Error handling:
  - API 500: Show `[data-testid="error-message"]` - "Failed to reschedule booking"
  - Keep modal open on error
  - Allow retry

#### FR-1.7: Success Feedback
- Display: `[data-testid="success-message"]` - "Booking rescheduled successfully"
- Auto-refresh booking list with new time
- Close modal automatically
- Clear success message after 5 seconds

### Technical Specifications

**Frontend (Elm)**:
```elm
type alias RescheduleOption =
    { time : String
    , reason : String
    , availabilityStatus : AvailabilityStatus
    , weatherStatus : WeatherStatus
    , isSelectable : Bool
    }

type AvailabilityStatus
    = Available
    | Unavailable

type WeatherStatus
    = Suitable
    | Marginal
    | NotSuitable
```

**Backend (Rust)**:
```rust
#[derive(Serialize, Deserialize)]
struct RescheduleRequest {
    new_scheduled_date: DateTime<Utc>,
    reason: RescheduleReason,
    ai_suggestion_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
enum RescheduleReason {
    Weather,
    InstructorUnavailable,
    StudentRequest,
}
```

**Database Migration**:
```sql
CREATE TABLE booking_reschedule_history (
    id TEXT PRIMARY KEY,
    booking_id TEXT NOT NULL REFERENCES bookings(id),
    old_date TIMESTAMP NOT NULL,
    new_date TIMESTAMP NOT NULL,
    reason TEXT NOT NULL,
    ai_suggestion_id TEXT,
    rescheduled_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    rescheduled_by TEXT
);
```

### Acceptance Criteria

- [ ] User can click reschedule button on any booking
- [ ] Modal displays 3 AI-generated options
- [ ] Each option shows time, reason, availability, and weather
- [ ] Unavailable options have disabled select buttons
- [ ] Confirmation dialog appears before rescheduling
- [ ] User can cancel at any point
- [ ] Loading state displays during API calls
- [ ] Success message appears on completion
- [ ] Booking list updates with new time
- [ ] Error messages display for API failures
- [ ] Reschedule history is logged

---

## Feature 2: Real-Time Weather Alert System

### Overview
Monitor weather conditions via WebSocket and display real-time alerts to prevent flight operations in dangerous conditions.

### User Stories

**US-2.1**: As a flight instructor, I want to see weather alerts as soon as they're issued so I can immediately assess impact on scheduled flights.

**US-2.2**: As a user, I want weather alerts to persist across page navigation so I don't miss critical safety information.

**US-2.3**: As a user, I want to dismiss weather alerts so I can clear my view once I've acknowledged them.

**US-2.4**: As a user, I want to see alert severity visually so I can prioritize my response.

**US-2.5**: As a user, I want dashboard stats to update with active alert counts so I have situational awareness.

### Functional Requirements

#### FR-2.1: Weather Alert Banner
- **Component**: WeatherAlertBanner.elm
- **Position**: Top of page, below header, above content
- **Element**: `[data-testid="weather-alert"]`
- **Display**:
  - Alert icon (⚠️ or similar)
  - Alert description
  - Location code (e.g., "KORD")
  - Timestamp
  - Dismiss button: `[data-testid="dismiss-alert-btn"]`

#### FR-2.2: Alert Severity Levels
Visual styling based on severity:

| Severity | CSS Class | Color | Icon |
|----------|-----------|-------|------|
| Severe | `.severe` | Red | ⛈️ |
| High | `.high` | Orange | 🌧️ |
| Moderate | `.moderate` | Yellow | ⚡ |
| Low | `.low` | Blue | 🌤️ |
| Clear | `.clear` | Green | ☀️ |

#### FR-2.3: Multiple Alert Handling
- Display all active alerts as stacked banners
- Maximum 5 alerts displayed simultaneously
- Oldest alerts auto-dismiss when limit exceeded
- Each alert independently dismissible

#### FR-2.4: WebSocket Message Format
```json
{
  "type": "weather_alert",
  "data": {
    "alert_id": "string",
    "booking_id": "string (optional)",
    "location": "KORD",
    "severity": "severe" | "high" | "moderate" | "low" | "clear",
    "description": "Thunderstorm warning",
    "timestamp": "2025-11-08T10:00:00Z",
    "expires_at": "2025-11-08T14:00:00Z",
    "affected_bookings": ["booking-id-1", "booking-id-2"]
  }
}
```

#### FR-2.5: Dashboard Stats Integration
- Update `[data-testid="stat-alerts"]` count when alerts received
- Decrement count when alerts dismissed
- Count persists across page navigation (stored in Elm model)

#### FR-2.6: Alert Persistence
- Alerts stored in Elm model state
- Persist during SPA navigation
- Clear on page reload (intentional - fresh alert check)
- WebSocket reconnection re-fetches active alerts

#### FR-2.7: Backend Weather Monitoring
- **Service**: WeatherMonitorService (Rust background task)
- **Frequency**: Check every 5 minutes
- **API**: OpenWeatherMap or NOAA API
- **Logic**:
  1. Fetch weather for all scheduled flight locations (next 24 hours)
  2. Detect dangerous conditions (wind, visibility, storms)
  3. Generate alerts for affected bookings
  4. Broadcast via WebSocket to all connected clients

### Technical Specifications

**Frontend (Elm)**:
```elm
type alias Alert =
    { id : String
    , bookingId : Maybe String
    , alertType : String
    , message : String
    , severity : Severity
    , location : String
    , timestamp : String
    , studentName : Maybe String
    , originalDate : Maybe String
    }

type Severity
    = Severe
    | High
    | Moderate
    | Low
    | Clear

-- Message handling
type Msg
    = WebSocketMessageReceived String
    | DismissAlert String
    | ...
```

**Backend (Rust)**:
```rust
#[derive(Serialize, Deserialize, Clone)]
struct WeatherAlert {
    alert_id: String,
    booking_id: Option<String>,
    location: String,
    severity: AlertSeverity,
    description: String,
    timestamp: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    affected_bookings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
enum AlertSeverity {
    Severe,
    High,
    Moderate,
    Low,
    Clear,
}

// Background service
async fn monitor_weather(db: Pool, ws_broadcaster: Broadcaster) {
    loop {
        let bookings = fetch_upcoming_bookings(&db, 24).await;
        for booking in bookings {
            let weather = fetch_weather(&booking.location).await;
            if let Some(alert) = analyze_weather_for_flight(weather, &booking) {
                save_alert(&db, &alert).await;
                ws_broadcaster.send_alert(alert).await;
            }
        }
        tokio::time::sleep(Duration::from_secs(300)).await; // 5 min
    }
}
```

### Acceptance Criteria

- [ ] Weather alerts display as banners at page top
- [ ] Alerts have severity-based color coding
- [ ] Multiple alerts display simultaneously
- [ ] Each alert shows location and timestamp
- [ ] User can dismiss individual alerts
- [ ] Dashboard stats reflect active alert count
- [ ] Alerts persist across page navigation
- [ ] Clear weather messages replace severe alerts
- [ ] Backend monitors weather every 5 minutes
- [ ] WebSocket broadcasts alerts to all clients

---

## Feature 3: Enhanced WebSocket Infrastructure

### Overview
Implement robust WebSocket connection management with automatic reconnection, status indicators, and resilient message handling.

### User Stories

**US-3.1**: As a user, I want to see the WebSocket connection status so I know if I'm receiving live updates.

**US-3.2**: As a user, I want automatic reconnection when the connection drops so I don't lose real-time features.

**US-3.3**: As a user, I want the app to continue working when WebSocket is disconnected so I'm not blocked from basic operations.

### Functional Requirements

#### FR-3.1: Connection Status Indicator
- **Element**: `[data-testid="ws-status"]` (already exists)
- **States**:
  - **Connecting**: "Connecting..." (yellow/gray dot)
  - **Connected**: "● Live" (green dot)
  - **Disconnected**: "○ Disconnected" (red/gray dot)

#### FR-3.2: Exponential Backoff Reconnection
- **Algorithm**:
  1. Attempt 1: 1 second delay
  2. Attempt 2: 2 seconds delay
  3. Attempt 3: 4 seconds delay
  4. Attempt 4: 8 seconds delay
  5. Attempt 5+: 16 seconds delay (max)
  6. Stop after 10 failed attempts, require manual refresh

#### FR-3.3: Connection Failure Handling
- Display status as "Disconnected"
- Show non-blocking notification: "Live updates unavailable. Reconnecting..."
- Allow all basic app functions (view bookings, create students, etc.)
- Disable features requiring real-time data (weather alerts, live notifications)

#### FR-3.4: Message Queue During Disconnect
- Buffer outgoing messages while disconnected
- Send queued messages on reconnection (up to 50 messages)
- Discard older messages if queue exceeds limit

#### FR-3.5: Heartbeat/Ping-Pong
- Client sends ping every 30 seconds
- Server responds with pong
- Client declares disconnected if no pong after 60 seconds

### Technical Specifications

**Frontend (Elm)**:
```elm
type WebSocketStatus
    = Connecting
    | Connected
    | Disconnected

type alias Model =
    { ...
    , websocketStatus : WebSocketStatus
    , reconnectAttempts : Int
    , messageQueue : List String
    }

-- Reconnection logic
reconnectWithBackoff : Int -> Cmd Msg
reconnectWithBackoff attempts =
    let
        delayMs = min 16000 (1000 * 2 ^ attempts)
    in
    Process.sleep delayMs
        |> Task.perform (\_ -> AttemptReconnect)
```

**Backend (Rust)**:
```rust
// WebSocket handler
async fn handle_websocket(
    ws: WebSocket,
    state: AppState,
    client_id: Uuid,
) {
    let (mut sender, mut receiver) = ws.split();

    // Ping/pong task
    let ping_task = tokio::spawn(async move {
        loop {
            if sender.send(Message::Ping(vec![])).await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });

    // Message receiver
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Pong(_) => continue,
            Message::Text(text) => {
                handle_client_message(&state, &client_id, text).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    ping_task.abort();
}
```

### Acceptance Criteria

- [ ] Status indicator shows "Connecting" on initial load
- [ ] Status updates to "Connected" when WebSocket connects
- [ ] Status shows "Disconnected" when connection fails
- [ ] Automatic reconnection attempts with exponential backoff
- [ ] App remains functional during disconnection
- [ ] Heartbeat pings sent every 30 seconds
- [ ] Connection declared dead after 60 seconds without pong
- [ ] Message queue buffers during disconnect
- [ ] Queued messages sent on reconnection

---

## Feature 4: Comprehensive Error Handling

### Overview
Gracefully handle all error scenarios with user-friendly messages and recovery options.

### User Stories

**US-4.1**: As a user, I want clear error messages when API calls fail so I understand what went wrong.

**US-4.2**: As a user, I want the app to recover from temporary network issues so I can retry without refreshing.

**US-4.3**: As a user, I want validation errors to guide me in fixing form data so I can complete my task.

### Functional Requirements

#### FR-4.1: API Error Display
- **Element**: `[data-testid="error-message"]`
- **Position**: Top of content area, below navigation
- **Styling**: Red background, white text, error icon
- **Auto-dismiss**: After 10 seconds (with manual close button)

#### FR-4.2: Error Message Mapping

| HTTP Status | User-Friendly Message | Technical Detail (dev mode) |
|-------------|----------------------|----------------------------|
| 400 | "Invalid request. Please check your input." | Bad Request |
| 401 | "Session expired. Please log in again." | Unauthorized |
| 403 | "You don't have permission for this action." | Forbidden |
| 404 | "Resource not found." | Not Found |
| 429 | "Too many requests. Please wait and try again." | Rate limit exceeded |
| 500 | "Server error. Our team has been notified." | Internal Server Error |
| 502 | "Service temporarily unavailable." | Bad Gateway |
| 503 | "Service temporarily unavailable." | Service Unavailable |
| Timeout | "Request timed out. Please check your connection." | Network timeout |
| Network | "Network error. Please check your connection." | Connection failed |

#### FR-4.3: Form Validation Errors
- **Display**: Below each form field
- **Element**: `[data-testid="error-{fieldname}"]`
- **Validation Rules**:

**Student Form**:
- Name: Required, min 2 characters
- Email: Required, valid email format
- Phone: Required
- Training Level: Required, must be valid enum value

**Booking Form**:
- Aircraft: Required, must be from allowed list
- Student: Required, must be existing student ID
- Start Time: Required, valid ISO 8601 datetime
- End Time: Required, valid datetime, must be after start time
- Location: Required, valid airport code (4-letter ICAO or 3-letter IATA)
- Coordinates: Optional, but if provided must be valid lat/lon

#### FR-4.4: Network Timeout Handling
- **Timeout**: 30 seconds for all API calls
- **Display**: `[data-testid="timeout-error"]`
- **Message**: "Request timed out. The server is taking too long to respond."
- **Action**: Retry button

#### FR-4.5: Rate Limiting
- **Client-side**: Debounce rapid requests (max 5 per second)
- **Server-side**: Return 429 after 100 requests per minute per IP
- **Display**: `[data-testid="error-message"]`
- **Message**: "Too many requests. Please wait 60 seconds and try again."
- **Auto-retry**: After 60 seconds

#### FR-4.6: Malformed Response Handling
- Detect invalid JSON
- Detect missing required fields
- Fallback message: "Received invalid data from server."
- Log error details for debugging

#### FR-4.7: Concurrent Request Handling
- Queue concurrent requests to same endpoint
- Execute sequentially to prevent race conditions
- Handle parallel requests to different endpoints independently
- Show loading state for all in-flight requests

### Technical Specifications

**Frontend (Elm)**:
```elm
type alias Model =
    { ...
    , error : Maybe String
    , pendingRequests : Dict String RequestStatus
    }

type RequestStatus
    = Pending
    | Success
    | Failed String

-- Error handling
handleHttpError : Http.Error -> String
handleHttpError error =
    case error of
        Http.BadUrl url ->
            "Invalid URL: " ++ url

        Http.Timeout ->
            "Request timed out. Please check your connection."

        Http.NetworkError ->
            "Network error. Please check your connection."

        Http.BadStatus 400 ->
            "Invalid request. Please check your input."

        Http.BadStatus 429 ->
            "Too many requests. Please wait and try again."

        Http.BadStatus 500 ->
            "Server error. Our team has been notified."

        Http.BadStatus code ->
            "Error " ++ String.fromInt code

        Http.BadBody message ->
            "Received invalid data: " ++ message
```

**Backend (Rust)**:
```rust
// Rate limiting middleware
async fn rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = get_client_ip(&req);
    let key = format!("rate_limit:{}", client_ip);

    let count: u32 = redis.incr(&key).await?;
    if count == 1 {
        redis.expire(&key, 60).await?;
    }

    if count > 100 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}

// Error response standardization
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
    details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Resource not found"
            ),
            AppError::ValidationError(details) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                &details
            ),
            // ... other errors
        };

        let body = Json(ErrorResponse {
            error: message.to_string(),
            code: code.to_string(),
            details: None,
        });

        (status, body).into_response()
    }
}
```

### Acceptance Criteria

- [ ] API errors display user-friendly messages
- [ ] HTTP status codes map to appropriate messages
- [ ] Form validation errors appear below fields
- [ ] Network timeouts show timeout-specific message
- [ ] Rate limiting prevents server overload
- [ ] Malformed responses don't crash the app
- [ ] Concurrent requests handled without conflicts
- [ ] Retry buttons available for transient errors
- [ ] Error messages auto-dismiss after 10 seconds
- [ ] Developer mode shows technical error details

---

## Feature 5: Loading States & User Feedback

### Overview
Provide visual feedback during all asynchronous operations to improve perceived performance and user confidence.

### User Stories

**US-5.1**: As a user, I want to see a loading indicator when submitting forms so I know my action is being processed.

**US-5.2**: As a user, I want submit buttons to be disabled during processing so I don't accidentally double-submit.

**US-5.3**: As a user, I want success messages after completing actions so I have confirmation.

### Functional Requirements

#### FR-5.1: Form Loading States
- **Element**: `[data-testid="loading-spinner"]`
- **Display**: Inline spinner next to submit button
- **Behavior**:
  - Show when form submitted
  - Hide when response received
  - Accompanied by button text change ("Create Student" → "Creating...")

#### FR-5.2: Button Disabled States
- Disable submit button when loading
- Disable button when form has validation errors
- Visual styling: Grayed out, reduced opacity, cursor: not-allowed

#### FR-5.3: Success Messages
- **Element**: `[data-testid="success-message"]`
- **Position**: Top of content area
- **Styling**: Green background, white text, checkmark icon
- **Messages**:
  - "Student created successfully"
  - "Booking created successfully"
  - "Booking rescheduled successfully"
  - "Changes saved successfully"
- **Duration**: Auto-dismiss after 5 seconds
- **Close button**: Manual dismiss option

#### FR-5.4: Page-Level Loading
- Initial page load: Full-page spinner
- Navigation: Fade transition with loading indicator
- Data refresh: Skeleton screens for list views

#### FR-5.5: Optimistic Updates
- Immediately add new items to lists (before API confirmation)
- Revert if API call fails
- Show subtle "saving..." indicator during sync

### Technical Specifications

**Frontend (Elm)**:
```elm
type alias Model =
    { ...
    , loading : Bool
    , successMessage : Maybe String
    }

-- Loading view
viewLoadingSpinner : Html Msg
viewLoadingSpinner =
    div [ class "loading-spinner", attribute "data-testid" "loading-spinner" ]
        [ div [ class "spinner" ] []
        , text "Loading..."
        ]

-- Success message with auto-dismiss
viewSuccessMessage : String -> Html Msg
viewSuccessMessage message =
    div
        [ class "success", attribute "data-testid" "success-message" ]
        [ text message
        , button
            [ class "success-dismiss"
            , onClick ClearSuccessMessage
            ]
            [ text "×" ]
        ]
```

**CSS**:
```css
.loading-spinner {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #f3f3f3;
  border-top: 2px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.success {
  background: #2ecc71;
  color: white;
  padding: 12px 16px;
  border-radius: 4px;
  margin-bottom: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
```

### Acceptance Criteria

- [ ] Loading spinner appears during form submission
- [ ] Submit buttons disable during processing
- [ ] Button text changes during loading
- [ ] Success messages appear after operations
- [ ] Success messages auto-dismiss after 5 seconds
- [ ] Users can manually close success messages
- [ ] Page transitions show loading indicators
- [ ] Optimistic updates provide instant feedback
- [ ] Failed optimistic updates revert cleanly

---

## Feature 6: Form Validation Enhancements

### Overview
Comprehensive client-side and server-side validation with clear, actionable error messages.

### User Stories

**US-6.1**: As a user, I want inline validation errors so I can fix issues as I type.

**US-6.2**: As a user, I want validation to prevent invalid submissions so I don't waste time.

**US-6.3**: As a user, I want specific error messages so I know exactly what to fix.

### Functional Requirements

#### FR-6.1: Real-Time Field Validation
- Validate on blur (user leaves field)
- Show error immediately below field
- Clear error when user corrects input

#### FR-6.2: Student Form Validation

| Field | Validations | Error Messages |
|-------|-------------|----------------|
| Name | Required, min 2 chars, max 100 chars | "Name is required", "Name must be at least 2 characters" |
| Email | Required, valid email format | "Email is required", "Please enter a valid email address" |
| Phone | Required, valid phone format | "Phone is required", "Please enter a valid phone number" |
| Training Level | Required, must be enum value | "Training level is required" |

#### FR-6.3: Booking Form Validation

| Field | Validations | Error Messages |
|-------|-------------|----------------|
| Aircraft Type | Required, must be from list | "Aircraft type is required" |
| Student | Required, existing student ID | "Student selection is required" |
| Start Time | Required, valid datetime, future | "Start time is required", "Start time must be in the future" |
| End Time | Required, valid datetime, after start | "End time is required", "End time must be after start time" |
| Location | Required, valid airport code | "Location is required", "Invalid airport code" |
| Latitude | Optional, valid range (-90 to 90) | "Latitude must be between -90 and 90" |
| Longitude | Optional, valid range (-180 to 180) | "Longitude must be between -180 and 180" |

#### FR-6.4: Server-Side Validation
- Duplicate all client-side validation on server
- Additional checks:
  - Aircraft availability (not double-booked)
  - Instructor availability
  - Student eligibility for aircraft type
  - Location exists in database

#### FR-6.5: Validation Error Display
- Field-level errors: `[data-testid="error-{fieldname}"]`
- Form-level errors: `[data-testid="form-errors"]`
- Scroll to first error on submit

### Technical Specifications

**Frontend (Elm)**:
```elm
type alias FormError =
    { field : String
    , message : String
    }

-- Validation functions
validateEmail : String -> Maybe String
validateEmail email =
    if String.isEmpty email then
        Just "Email is required"
    else if not (String.contains "@" email) then
        Just "Please enter a valid email address"
    else
        Nothing

validateDateRange : String -> String -> List FormError
validateDateRange startTime endTime =
    let
        start = Iso8601.toTime startTime
        end = Iso8601.toTime endTime
    in
    case (start, end) of
        (Ok s, Ok e) ->
            if Time.posixToMillis e <= Time.posixToMillis s then
                [ { field = "end-time"
                  , message = "End time must be after start time"
                  }
                ]
            else
                []

        _ ->
            []
```

**Backend (Rust)**:
```rust
#[derive(Debug, Validate)]
struct CreateStudentRequest {
    #[validate(length(min = 2, max = 100))]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(phone)]
    phone: String,

    #[validate(custom = "validate_training_level")]
    training_level: String,
}

fn validate_training_level(level: &str) -> Result<(), ValidationError> {
    match level {
        "STUDENT_PILOT" | "PRIVATE_PILOT" | "INSTRUMENT_RATED" | "COMMERCIAL_PILOT" => Ok(()),
        _ => Err(ValidationError::new("Invalid training level")),
    }
}
```

### Acceptance Criteria

- [ ] Validation errors appear on field blur
- [ ] Errors clear when user corrects input
- [ ] Submit button disabled when validation errors exist
- [ ] Server validates all inputs
- [ ] Server returns specific validation errors
- [ ] Form scrolls to first error on submit
- [ ] All error messages are user-friendly
- [ ] Required fields marked with asterisk

---

## Non-Functional Requirements

### Performance
- **Page Load**: < 2 seconds on 3G connection
- **API Response**: < 500ms for 95th percentile
- **WebSocket Latency**: < 100ms for real-time updates
- **Form Submission**: < 1 second feedback

### Scalability
- Support 1000+ concurrent WebSocket connections
- Handle 100+ bookings per flight school
- Rate limiting: 100 requests/minute per user

### Security
- All API endpoints require authentication (future)
- Input validation on client and server
- SQL injection prevention
- XSS prevention in all user inputs
- CORS properly configured

### Accessibility
- WCAG 2.1 Level AA compliance
- Keyboard navigation for all features
- Screen reader support
- Focus indicators on all interactive elements
- Error messages announced to screen readers

### Browser Support
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

---

## Success Metrics

### User Experience
- **Task Completion Rate**: > 95% for reschedule flow
- **Error Recovery Rate**: > 90% of users recover from errors without support
- **Time to Reschedule**: < 2 minutes from decision to confirmation

### Technical Performance
- **API Error Rate**: < 1% of requests
- **WebSocket Uptime**: > 99.5%
- **Mean Time to Recovery**: < 5 seconds after connection drop

### Business Impact
- **Weather-Related Cancellations**: Reduce by 30% through proactive rescheduling
- **Support Tickets**: Reduce error-related tickets by 50%
- **User Satisfaction**: NPS > 40

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-3)
- [ ] Enhanced WebSocket infrastructure
- [ ] Connection status indicators
- [ ] Automatic reconnection with backoff
- [ ] Basic error handling framework

### Phase 2: User Feedback (Weeks 3-5)
- [ ] Loading states for all async operations
- [ ] Success/error message components
- [ ] Form validation enhancements
- [ ] Comprehensive error messages

### Phase 3: Weather Alerts (Weeks 5-7)
- [ ] Weather alert banner component
- [ ] WebSocket alert broadcasting
- [ ] Backend weather monitoring service
- [ ] Alert persistence and dismissal

### Phase 4: Reschedule System (Weeks 7-12)
- [ ] Reschedule modal UI
- [ ] OpenAI integration for suggestions
- [ ] Instructor availability checking
- [ ] Weather suitability indicators
- [ ] Backend reschedule API
- [ ] Confirmation workflow
- [ ] Reschedule history logging

### Phase 5: Testing & Refinement (Weeks 12-14)
- [ ] E2E test suite completion
- [ ] Performance optimization
- [ ] Security audit
- [ ] Accessibility review
- [ ] User acceptance testing

---

## Dependencies

### External Services
- **OpenAI API**: For AI-powered reschedule suggestions
- **OpenWeatherMap API**: For real-time weather data
- **WebSocket Infrastructure**: Requires stable WebSocket support

### Internal Dependencies
- Elm frontend framework
- Rust backend with Axum
- SQLite database (migration to PostgreSQL recommended for production)
- Redis (for rate limiting and caching)

---

## Risks & Mitigation

### Risk 1: OpenAI API Costs
- **Mitigation**: Cache suggestions, limit to 3 options, implement usage monitoring

### Risk 2: Weather API Rate Limits
- **Mitigation**: Cache weather data, batch requests, fallback to manual entry

### Risk 3: WebSocket Scalability
- **Mitigation**: Load testing, horizontal scaling plan, WebSocket cluster

### Risk 4: Complex Reschedule Logic
- **Mitigation**: Phased rollout, extensive testing, manual override capability

---

## Open Questions

1. Should we support bulk rescheduling (reschedule multiple bookings at once)?
2. What permissions model for rescheduling (student vs. instructor)?
3. Should weather alerts trigger automatic notifications (email/SMS)?
4. How long should reschedule history be retained?
5. Should we integrate with instructor calendar systems (Google Calendar, Outlook)?

---

## Appendix: Test Coverage Gaps

The following E2E tests are currently failing and drive the requirements in this PRD:

### Reschedule Flow Tests
- [ ] should display 3 AI-suggested reschedule options
- [ ] should show instructor availability badges on reschedule options
- [ ] should allow selection of reschedule option and update booking
- [ ] should prevent selection of unavailable instructor slots
- [ ] should show weather suitability indicators
- [ ] should handle reschedule API errors gracefully
- [ ] should show loading state during reschedule operation
- [ ] should allow cancellation of reschedule operation

### Weather Alerts Tests
- [ ] should display real-time weather alert banner via WebSocket
- [ ] should handle multiple simultaneous weather alerts
- [ ] should allow weather alert dismissal
- [ ] should update dashboard stats when weather alerts are received
- [ ] should handle weather alert with clear weather message
- [ ] should persist weather alerts across page navigation
- [ ] should show appropriate styling for different alert severities

### Error Scenarios Tests
- [ ] should handle API 500 errors gracefully
- [ ] should handle network timeouts
- [ ] should handle form validation for invalid coordinates
- [ ] should handle WebSocket connection failures
- [ ] should handle malformed API responses
- [ ] should handle API rate limiting
- [ ] should handle concurrent API calls without conflicts
- [ ] should handle invalid form data submission
- [ ] should recover from temporary API outages

### Loading States Tests
- [ ] should show loading states during form submission
- [ ] should disable submit buttons during processing
- [ ] should show loading spinner during data fetch

### Success Feedback Tests
- [ ] should display success messages after creating students
- [ ] should display success messages after creating bookings
- [ ] should auto-dismiss success messages after 5 seconds

---

**Document Version History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-11-08 | Engineering Team | Initial PRD based on E2E test analysis |

---

**Sign-off**

This document requires approval from:
- [ ] Product Owner
- [ ] Engineering Lead
- [ ] UX/UI Designer
- [ ] QA Lead
