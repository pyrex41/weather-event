# Task Master Review: Next Phase (tag=next)
**Flight Schedule Pro - Phase 2 Implementation Tasks**

## Document Control
- **Review Date**: 2025-11-08
- **Total Tasks**: 10
- **Status**: All Pending
- **Priority**: All Medium
- **Related PRD**: [prd-next.md](./prd-next.md)
- **Complexity Analysis**: [task-complexity-report_next.json](../reports/task-complexity-report_next.json)

---

## Executive Summary

The "next" phase contains 10 interconnected tasks focused on enhancing the Flight Schedule Pro application with real-time features, robust error handling, and AI-powered scheduling capabilities. This review analyzes each task's complexity, dependencies, alignment with the PRD, and implementation recommendations.

### Key Insights

**High Complexity Tasks (Score ≥ 7)**:
- Task 1: Enhance WebSocket Infrastructure (Score: 8)
- Task 7: Build Backend Weather Monitoring Service (Score: 7)
- Task 9: Integrate OpenAI API (Score: 7)
- Task 10: Implement Backend Reschedule API (Score: 7)

**Critical Path**: 1 → 2 → 6 → 7 (WebSocket foundation → Weather system)
**Secondary Path**: 3 → 4 → 8 → 9 → 10 (Error handling → Reschedule system)

**Total Recommended Subtasks**: 12 (for the 6 complex tasks)

---

## Task Breakdown & Analysis

### 🔴 Task 1: Enhance WebSocket Infrastructure
**Complexity Score**: 8/10 | **Recommended Subtasks**: 4

#### Description
Implement robust WebSocket connection management with automatic reconnection, status indicators, and resilient message handling.

#### Why This Is Complex
- **Cross-language implementation**: Requires coordinated changes in both Elm (frontend) and Rust (backend)
- **Async handling**: Uses tokio for async operations, requires careful state management
- **Exponential backoff algorithm**: Non-trivial reconnection logic
- **Comprehensive testing**: Unit, integration, and E2E tests across the stack

#### Recommended Subtask Breakdown
1. **Elm WebSocket Module Extensions**
   - Implement connection state tracking (Connecting, Connected, Disconnected)
   - Add exponential backoff reconnection logic (1s, 2s, 4s, 8s, 16s max)
   - Create message queuing for offline scenarios (buffer up to 50 messages)
   - Handle cleanup and resource management

2. **Rust Backend Handler Updates**
   - Implement ping/pong heartbeat mechanism (30s intervals)
   - Add disconnection detection (60s timeout without pong)
   - Enhance client connection management and tracking
   - Implement graceful shutdown handling

3. **Integration Testing**
   - Test reconnection scenarios (network drop, server restart)
   - Verify message delivery guarantees
   - Test concurrent connections (load testing)
   - Validate cleanup on disconnection

4. **E2E Testing for Resilience**
   - Test connection failure recovery
   - Verify status indicator updates
   - Test message queuing and delivery
   - Validate app functionality during disconnect

#### Dependencies
None (foundation task)

#### PRD Alignment
✅ Directly implements **Feature 3: Enhanced WebSocket Infrastructure** from PRD
- FR-3.1: Connection Status Indicator
- FR-3.2: Exponential Backoff Reconnection
- FR-3.3: Connection Failure Handling
- FR-3.4: Message Queue During Disconnect
- FR-3.5: Heartbeat/Ping-Pong

#### Implementation Considerations

**Elm Frontend**:
```elm
type WebSocketStatus
    = Connecting
    | Connected
    | Disconnected

type alias Model =
    { websocketStatus : WebSocketStatus
    , reconnectAttempts : Int
    , messageQueue : List String
    , lastPingTime : Time.Posix
    }

-- Exponential backoff calculation
getReconnectDelay : Int -> Int
getReconnectDelay attempts =
    min 16000 (1000 * 2 ^ attempts)
```

**Rust Backend**:
```rust
struct WebSocketClient {
    id: Uuid,
    sender: SplitSink<WebSocket, Message>,
    last_pong: Arc<Mutex<Instant>>,
}

async fn heartbeat_task(client_id: Uuid, state: AppState) {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        // Check last pong time
        let last = state.get_client_last_pong(&client_id).await;
        if last.elapsed() > Duration::from_secs(60) {
            state.disconnect_client(&client_id).await;
            break;
        }

        // Send ping
        state.send_ping(&client_id).await;
    }
}
```

#### Testing Strategy
- **Unit Tests**: Backoff calculation, state transitions, message queuing
- **Integration Tests**: Mock WebSocket server, test reconnection sequences
- **E2E Tests**: Full stack testing with network simulation

#### Estimated Effort
**3-4 weeks** (1 week per subtask)

---

### 🟡 Task 2: Implement Connection Status Indicator
**Complexity Score**: 4/10 | **Recommended Subtasks**: 0

#### Description
Add a visual WebSocket connection status indicator to inform users of real-time update availability.

#### Why This Is Moderate
- Primarily UI-focused work in Elm
- Reactive to model state changes (simple event-driven pattern)
- Depends on Task 1 for state management
- No complex business logic

#### Implementation Details
Already partially implemented! The element `[data-testid="ws-status"]` exists in Main.elm:

```elm
viewWebSocketStatus : WebSocketStatus -> Html Msg
viewWebSocketStatus status =
    case status of
        Connecting ->
            span [ class "status-badge connecting", attribute "data-testid" "ws-status" ]
                [ text "Connecting..." ]
        Connected ->
            span [ class "status-badge connected", attribute "data-testid" "ws-status" ]
                [ text "● Live" ]
        Disconnected ->
            span [ class "status-badge disconnected", attribute "data-testid" "ws-status" ]
                [ text "○ Disconnected" ]
```

#### Remaining Work
- ✅ Status element exists
- ⚠️ Need to wire up state changes from Task 1
- ⚠️ Add CSS animations for status transitions
- ⚠️ Add tooltip with connection details (uptime, last ping)
- ⚠️ Add click handler for manual reconnect

#### Dependencies
- Task 1 (WebSocket infrastructure must be complete)

#### PRD Alignment
✅ Implements FR-3.1: Connection Status Indicator

#### Estimated Effort
**1 week** (mostly CSS and event wiring)

---

### 🔴 Task 3: Develop Comprehensive Error Handling System
**Complexity Score**: 6/10 | **Recommended Subtasks**: 2

#### Description
Implement user-friendly error messages, recovery options, and handling for all error scenarios including API failures, timeouts, and validation.

#### Why This Is Complex
- **Dual implementation**: Both Elm frontend and Rust backend
- **Middleware integration**: Redis for rate limiting
- **Consistent error formats**: Requires standardization across the stack
- **Edge case handling**: Timeout, network errors, malformed responses

#### Recommended Subtask Breakdown
1. **Elm Error Handling Module**
   - Create centralized error message mapping (HTTP status → user message)
   - Implement error display component with auto-dismiss
   - Add retry logic for transient errors
   - Handle malformed JSON gracefully

2. **Rust Backend Error Standardization**
   - Create unified ErrorResponse struct
   - Implement IntoResponse for all error types
   - Add rate limiting middleware (Redis-based)
   - Add request/response logging for debugging

#### Dependencies
None (foundation task)

#### PRD Alignment
✅ Directly implements **Feature 4: Comprehensive Error Handling** from PRD
- FR-4.1: API Error Display
- FR-4.2: Error Message Mapping
- FR-4.4: Network Timeout Handling
- FR-4.5: Rate Limiting
- FR-4.6: Malformed Response Handling

#### Implementation Details

**Error Message Mapping Table** (from PRD):

| HTTP Status | User Message | Technical Detail |
|-------------|--------------|------------------|
| 400 | "Invalid request. Please check your input." | Bad Request |
| 429 | "Too many requests. Please wait and try again." | Rate limit |
| 500 | "Server error. Our team has been notified." | Internal error |
| Timeout | "Request timed out. Please check your connection." | Network timeout |

**Elm Error Handler**:
```elm
handleHttpError : Http.Error -> String
handleHttpError error =
    case error of
        Http.BadStatus 429 ->
            "Too many requests. Please wait and try again."
        Http.Timeout ->
            "Request timed out. Please check your connection."
        Http.NetworkError ->
            "Network error. Please check your connection."
        Http.BadStatus code ->
            "Error " ++ String.fromInt code
        Http.BadBody message ->
            "Received invalid data from server."
```

**Rust Error Types**:
```rust
#[derive(Debug, Serialize)]
pub enum AppError {
    NotFound,
    ValidationError(String),
    DatabaseError(sqlx::Error),
    ExternalApiError(String),
    RateLimitExceeded,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Resource not found"
            ),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT",
                "Too many requests"
            ),
            // ... other mappings
        };

        (status, Json(ErrorResponse {
            error: message.to_string(),
            code: code.to_string()
        })).into_response()
    }
}
```

#### Testing Strategy
- Test each HTTP status code mapping
- Test rate limiting (exceed threshold, verify 429)
- Test timeout scenarios (mock slow responses)
- Test malformed JSON handling

#### Estimated Effort
**2-3 weeks** (1.5 weeks per subtask)

---

### 🟡 Task 4: Add Loading States and User Feedback Components
**Complexity Score**: 5/10 | **Recommended Subtasks**: 0

#### Description
Provide visual feedback during asynchronous operations with loading indicators, disabled buttons, and success messages.

#### Why This Is Moderate
- Primarily UI patterns in Elm
- Dependent on Task 3 for error framework
- Straightforward implementation (spinners, button states)
- Requires integration across multiple forms

#### Implementation Details

**Current State Analysis**:
✅ Loading spinner element exists: `[data-testid="loading-spinner"]`
❌ Not consistently used across all forms
❌ Success messages not auto-dismissing
❌ Optimistic updates not implemented

**Loading Spinner Component**:
```elm
viewLoadingSpinner : Html Msg
viewLoadingSpinner =
    div [ class "loading-spinner", attribute "data-testid" "loading-spinner" ]
        [ div [ class "spinner" ] []
        , text "Loading..."
        ]
```

**Success Message with Auto-Dismiss**:
```elm
type Msg
    = ...
    | ClearSuccessMessage
    | ScheduleClearSuccessMessage

update msg model =
    case msg of
        StudentCreated (Ok student) ->
            ( { model
                | students = student :: model.students
                , successMessage = Just "Student created successfully"
                , loading = False
              }
            , Task.perform (\_ -> ScheduleClearSuccessMessage)
                (Process.sleep 5000)
            )

        ScheduleClearSuccessMessage ->
            ( model, Task.perform (\_ -> ClearSuccessMessage) (Task.succeed ()) )

        ClearSuccessMessage ->
            ( { model | successMessage = Nothing }, Cmd.none )
```

#### Dependencies
- Task 3 (Error handling framework)

#### PRD Alignment
✅ Implements **Feature 5: Loading States & User Feedback** from PRD
- FR-5.1: Form Loading States
- FR-5.2: Button Disabled States
- FR-5.3: Success Messages
- FR-5.4: Page-Level Loading
- FR-5.5: Optimistic Updates

#### Estimated Effort
**2 weeks**

---

### 🔴 Task 5: Enhance Form Validation
**Complexity Score**: 6/10 | **Recommended Subtasks**: 2

#### Description
Implement comprehensive client-side and server-side validation with real-time feedback and actionable error messages.

#### Why This Is Complex
- **Dual-side validation**: Client (Elm) and server (Rust) must match
- **Real-time UX**: Validate on blur, show immediate feedback
- **Complex rules**: Email format, phone format, date ranges, airport codes
- **Integration**: Must work with error handling system (Task 3)

#### Recommended Subtask Breakdown
1. **Elm Client-Side Validation**
   - Implement validation functions for each field type
   - Add blur event handlers for real-time feedback
   - Create field error display components
   - Scroll to first error on submit

2. **Rust Server-Side Validation**
   - Use `validator` crate for common validations
   - Implement custom validators (airport codes, training levels)
   - Add business logic checks (aircraft availability, date conflicts)
   - Return structured validation errors

#### Current Validation Gaps

**Student Form** (partially implemented):
- ✅ Name: Required, min 2 chars
- ✅ Email: Required, contains "@"
- ✅ Phone: Required
- ✅ Training Level: Required
- ❌ Email: Full RFC 5322 validation
- ❌ Phone: International format validation
- ❌ Name: Max length, special character handling

**Booking Form** (missing):
- ❌ Start time: Must be in future
- ❌ End time: Must be after start time
- ❌ Location: Valid airport code (ICAO/IATA)
- ❌ Coordinates: Valid lat/lon ranges
- ❌ Aircraft: Must be from allowed list

#### Implementation Examples

**Elm Validators**:
```elm
validateEmail : String -> Maybe String
validateEmail email =
    if String.isEmpty email then
        Just "Email is required"
    else if not (Regex.contains emailRegex email) then
        Just "Please enter a valid email address"
    else
        Nothing

validateDateRange : String -> String -> List FormError
validateDateRange startTime endTime =
    case (Iso8601.toTime startTime, Iso8601.toTime endTime) of
        (Ok start, Ok end) ->
            if Time.posixToMillis end <= Time.posixToMillis start then
                [{ field = "end-time", message = "End time must be after start time" }]
            else
                []
        _ ->
            [{ field = "start-time", message = "Invalid date format" }]
```

**Rust Validators**:
```rust
#[derive(Validate)]
struct CreateBookingRequest {
    #[validate(custom = "validate_airport_code")]
    location: String,

    #[validate(custom = "validate_future_date")]
    start_time: DateTime<Utc>,

    #[validate(custom = "validate_date_range")]
    end_time: DateTime<Utc>,
}

fn validate_airport_code(code: &str) -> Result<(), ValidationError> {
    if code.len() == 3 || code.len() == 4 {
        if code.chars().all(|c| c.is_ascii_alphabetic()) {
            return Ok(());
        }
    }
    Err(ValidationError::new("Invalid airport code"))
}
```

#### Dependencies
- Task 3 (Error handling for displaying validation errors)

#### PRD Alignment
✅ Implements **Feature 6: Form Validation Enhancements** from PRD
- FR-6.1: Real-Time Field Validation
- FR-6.2: Student Form Validation
- FR-6.3: Booking Form Validation
- FR-6.4: Server-Side Validation
- FR-6.5: Validation Error Display

#### Estimated Effort
**2-3 weeks** (1.5 weeks per subtask)

---

### 🟡 Task 6: Implement Weather Alert Banner and Display Logic
**Complexity Score**: 5/10 | **Recommended Subtasks**: 0

#### Description
Create a component to display real-time weather alerts with severity-based styling and dismissal functionality.

#### Why This Is Moderate
- Focused on Elm UI component
- WebSocket message handling (depends on Task 1)
- State management for multiple alerts
- Persistence across navigation

#### Implementation Details

**Alert Model** (already defined in Types.elm):
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
```

**Alert Banner Component** (needs creation):
```elm
viewWeatherAlert : Alert -> Html Msg
viewWeatherAlert alert =
    div
        [ class ("alert alert-" ++ severityClass alert.severity)
        , attribute "data-testid" "weather-alert"
        ]
        [ span [ class "alert-icon" ] [ text (severityIcon alert.severity) ]
        , div [ class "alert-content" ]
            [ strong [] [ text alert.alertType ]
            , text (" - " ++ alert.message)
            , text (" at " ++ alert.location)
            ]
        , button
            [ class "alert-dismiss"
            , onClick (DismissAlert alert.id)
            , attribute "data-testid" "dismiss-alert-btn"
            ]
            [ text "×" ]
        ]

severityClass : Severity -> String
severityClass severity =
    case severity of
        Severe -> "severe"
        High -> "high"
        Moderate -> "moderate"
        Low -> "low"
        Clear -> "clear"

severityIcon : Severity -> String
severityIcon severity =
    case severity of
        Severe -> "⛈️"
        High -> "🌧️"
        Moderate -> "⚡"
        Low -> "🌤️"
        Clear -> "☀️"
```

**WebSocket Message Handler** (partially implemented):
```elm
WebSocketMessageReceived message ->
    case Decode.decodeString alertDecoder message of
        Ok alert ->
            ( { model | alerts = alert :: model.alerts }, Cmd.none )
        Err _ ->
            ( model, Cmd.none )
```

**Remaining Work**:
- ✅ Alert model exists
- ✅ WebSocket message handling exists
- ❌ Alert banner UI component
- ❌ Severity-based CSS styling
- ❌ Multiple alert display (max 5)
- ❌ Dashboard stats integration
- ❌ Alert persistence across navigation (already works via model)

#### Dependencies
- Task 1 (WebSocket infrastructure for receiving alerts)

#### PRD Alignment
✅ Partially implements **Feature 2: Real-Time Weather Alert System** from PRD
- FR-2.1: Weather Alert Banner
- FR-2.2: Alert Severity Levels
- FR-2.3: Multiple Alert Handling
- FR-2.4: WebSocket Message Format (decoder exists)
- FR-2.5: Dashboard Stats Integration
- FR-2.6: Alert Persistence

#### Estimated Effort
**1.5 weeks**

---

### 🔴 Task 7: Build Backend Weather Monitoring Service
**Complexity Score**: 7/10 | **Recommended Subtasks**: 2

#### Description
Develop a background service to monitor weather conditions and generate alerts for affected bookings.

#### Why This Is Complex
- **Async background service**: Requires tokio spawn and proper lifecycle management
- **External API integration**: OpenWeatherMap or NOAA API
- **Data analysis**: Determine if weather is safe for flight based on training level
- **Broadcasting**: Send alerts via WebSocket to all connected clients
- **Testing challenges**: Mock external APIs, test alert logic

#### Recommended Subtask Breakdown
1. **Weather Data Fetching and Analysis**
   - Integrate with OpenWeatherMap API
   - Implement weather parsing and data extraction
   - Create flight safety analysis logic (visibility, wind, ceiling, thunderstorms)
   - Cache weather data to reduce API calls

2. **Alert Generation and Broadcasting**
   - Query database for upcoming bookings (next 24 hours)
   - Generate alerts for unsafe conditions
   - Store alerts in database
   - Broadcast via WebSocket to all connected clients
   - Implement scheduling (every 5 minutes)

#### Implementation Details

**Weather API Integration**:
```rust
#[derive(Deserialize)]
struct WeatherResponse {
    visibility: f64,
    wind: WindData,
    weather: Vec<WeatherCondition>,
    clouds: CloudData,
}

async fn fetch_weather(location: &Location) -> Result<WeatherResponse> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}",
        location.lat, location.lon, API_KEY
    );

    let response = reqwest::get(&url).await?.json().await?;
    Ok(response)
}
```

**Flight Safety Analysis**:
```rust
fn is_safe_to_fly(
    weather: &WeatherResponse,
    minimums: &WeatherMinimum,
) -> (bool, Option<String>) {
    // Check visibility
    let visibility_sm = weather.visibility * 0.000621371; // m to miles
    if visibility_sm < minimums.min_visibility_sm {
        return (false, Some(format!("Visibility {} SM below minimum {} SM",
            visibility_sm, minimums.min_visibility_sm)));
    }

    // Check wind speed
    let wind_kt = weather.wind.speed * 1.94384; // m/s to knots
    if wind_kt > minimums.max_wind_speed_kt {
        return (false, Some(format!("Wind {} kt exceeds maximum {} kt",
            wind_kt, minimums.max_wind_speed_kt)));
    }

    // Check ceiling (if required)
    if let Some(min_ceiling) = minimums.min_ceiling_ft {
        let ceiling_ft = estimate_ceiling(&weather.clouds);
        if ceiling_ft < min_ceiling {
            return (false, Some(format!("Ceiling {} ft below minimum {} ft",
                ceiling_ft, min_ceiling)));
        }
    }

    // Check thunderstorms
    if minimums.no_thunderstorms {
        if weather.weather.iter().any(|w| w.id >= 200 && w.id < 300) {
            return (false, Some("Thunderstorms in area".to_string()));
        }
    }

    (true, None)
}
```

**Background Service**:
```rust
pub async fn start_weather_monitor(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 min

        loop {
            interval.tick().await;

            if let Err(e) = check_weather_for_bookings(&state).await {
                tracing::error!("Weather check failed: {}", e);
            }
        }
    });
}

async fn check_weather_for_bookings(state: &AppState) -> Result<()> {
    // Get bookings for next 24 hours
    let bookings = get_upcoming_bookings(&state.db, 24).await?;

    for booking in bookings {
        let weather = fetch_weather(&booking.departure_location).await?;
        let minimums = get_weather_minimums(&state.db, &booking.student_id).await?;

        let (is_safe, reason) = is_safe_to_fly(&weather, &minimums);

        if !is_safe {
            let alert = create_alert(&booking, &weather, reason);
            save_alert(&state.db, &alert).await?;
            broadcast_alert(&state.ws_broadcaster, &alert).await?;
        }
    }

    Ok(())
}
```

#### Dependencies
- Task 6 (Alert banner must exist to display alerts)

#### PRD Alignment
✅ Implements **Feature 2: Real-Time Weather Alert System** (backend portion)
- FR-2.7: Backend Weather Monitoring

#### Estimated Effort
**3 weeks** (1.5 weeks per subtask)

---

### 🟡 Task 8: Create Reschedule Modal UI with Options Display
**Complexity Score**: 5/10 | **Recommended Subtasks**: 0

#### Description
Build the UI for the reschedule modal, displaying AI-suggested options with availability and weather indicators.

#### Why This Is Moderate
- Elm UI component work
- Well-defined structure (modal with 3 option cards)
- Depends on error handling (Task 3) and loading states (Task 4)
- No complex business logic in this task

#### Implementation Structure

**Reschedule Modal Component**:
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

viewRescheduleModal : Model -> Maybe Booking -> Html Msg
viewRescheduleModal model maybeBooking =
    case maybeBooking of
        Nothing ->
            text ""

        Just booking ->
            div [ class "modal-overlay", attribute "data-testid" "reschedule-modal" ]
                [ div [ class "modal-content reschedule-modal" ]
                    [ h2 [] [ text "Reschedule Flight" ]
                    , viewOriginalBooking booking
                    , viewRescheduleOptions model.rescheduleOptions
                    , button [ class "button-secondary", onClick CloseRescheduleModal ]
                        [ text "Cancel" ]
                    ]
                ]

viewRescheduleOptions : List RescheduleOption -> Html Msg
viewRescheduleOptions options =
    div [ class "reschedule-options" ]
        (List.indexedMap viewRescheduleOption options)

viewRescheduleOption : Int -> RescheduleOption -> Html Msg
viewRescheduleOption index option =
    div [ class "reschedule-option", attribute "data-testid" "reschedule-option" ]
        [ div [ class "option-time", attribute "data-testid" "option-time" ]
            [ text option.time ]
        , div [ class "option-reason", attribute "data-testid" "option-reason" ]
            [ text option.reason ]
        , viewAvailabilityBadge option.availabilityStatus
        , viewWeatherIndicator option.weatherStatus
        , button
            [ class "button-primary"
            , onClick (SelectRescheduleOption index)
            , disabled (not option.isSelectable)
            , attribute "data-testid" "select-option-btn"
            ]
            [ text "Select" ]
        ]

viewAvailabilityBadge : AvailabilityStatus -> Html Msg
viewAvailabilityBadge status =
    case status of
        Available ->
            span [ class "badge available", attribute "data-testid" "availability-badge" ]
                [ text "Available" ]

        Unavailable ->
            span [ class "badge unavailable", attribute "data-testid" "availability-badge" ]
                [ text "Unavailable" ]

viewWeatherIndicator : WeatherStatus -> Html Msg
viewWeatherIndicator status =
    case status of
        Suitable ->
            span [ class "weather-indicator suitable", attribute "data-testid" "weather-indicator" ]
                [ text "☀️ Weather OK" ]

        Marginal ->
            span [ class "weather-indicator marginal", attribute "data-testid" "weather-indicator" ]
                [ text "⚡ Marginal" ]

        NotSuitable ->
            span [ class "weather-indicator unsuitable", attribute "data-testid" "weather-indicator" ]
                [ text "⛈️ Not Suitable" ]
```

**Confirmation Modal**:
```elm
viewConfirmRescheduleModal : Model -> Maybe RescheduleOption -> Html Msg
viewConfirmRescheduleModal model maybeOption =
    case maybeOption of
        Nothing ->
            text ""

        Just option ->
            div [ class "modal-overlay", attribute "data-testid" "confirm-reschedule-modal" ]
                [ div [ class "modal-content" ]
                    [ h3 [] [ text "Confirm Reschedule" ]
                    , p [] [ text ("New time: " ++ option.time) ]
                    , p [] [ text ("Reason: " ++ option.reason) ]
                    , div [ class "button-group" ]
                        [ button
                            [ class "button-primary"
                            , onClick ConfirmReschedule
                            , disabled model.loading
                            , attribute "data-testid" "confirm-reschedule-btn"
                            ]
                            [ text "Confirm" ]
                        , button
                            [ class "button-secondary"
                            , onClick CancelReschedule
                            , attribute "data-testid" "cancel-reschedule-btn"
                            ]
                            [ text "Cancel" ]
                        ]
                    , if model.loading then
                        viewLoadingSpinner
                      else
                        text ""
                    ]
                ]
```

#### Dependencies
- Task 3 (Error handling)
- Task 4 (Loading states)

#### PRD Alignment
✅ Implements **Feature 1: AI-Powered Reschedule System** (UI portion)
- FR-1.1: Reschedule Modal UI
- FR-1.2: Reschedule Options Display
- FR-1.4: Confirmation Dialog
- FR-1.6: Loading and Error States

#### Estimated Effort
**2 weeks**

---

### 🔴 Task 9: Integrate OpenAI API for AI-Powered Suggestions
**Complexity Score**: 7/10 | **Recommended Subtasks**: 2

#### Description
Connect to OpenAI API to generate intelligent reschedule options based on booking details and constraints.

#### Why This Is Complex
- **External API integration**: Requires API key management, error handling
- **Prompt engineering**: Craft effective prompts for consistent, useful suggestions
- **Response parsing**: Parse and validate JSON from AI responses
- **Caching**: Implement caching to reduce costs
- **Testing**: Mock API responses for testing

#### Recommended Subtask Breakdown
1. **API Service Implementation**
   - Set up OpenAI client with API key from environment
   - Design prompt template system
   - Implement request/response types
   - Parse AI responses into RescheduleOption structs

2. **Caching and Error Handling**
   - Implement Redis caching for suggestions (TTL: 1 hour)
   - Handle API errors gracefully (fallback to rule-based suggestions)
   - Implement rate limiting for OpenAI calls
   - Add usage monitoring and logging

#### Implementation Details

**Prompt Template**:
```rust
fn build_reschedule_prompt(
    booking: &Booking,
    student: &Student,
    weather_forecast: &WeatherForecast,
    instructor_availability: &[TimeSlot],
) -> String {
    format!(r#"
You are a flight scheduling assistant. Generate 3 alternative time slots for a rescheduled flight.

Current Booking:
- Date/Time: {}
- Student: {} (Training Level: {:?})
- Aircraft: {}
- Location: {}

Reason for Reschedule: Weather conditions unsafe
Weather Forecast (next 7 days): {}
Instructor Availability: {}

Requirements:
1. Suggest 3 alternative time slots within the next 7 days
2. Consider weather suitability for VFR flight
3. Prefer morning flights (better weather conditions)
4. Include reasoning for each suggestion

Response Format (JSON):
{{
  "suggestions": [
    {{
      "datetime": "2025-11-09T09:00:00Z",
      "reason": "Clear skies forecast, morning light winds",
      "confidence": "high"
    }},
    ...
  ]
}}
"#,
        booking.scheduled_date,
        student.name,
        student.training_level,
        booking.aircraft_type,
        booking.departure_location.name,
        format_weather_forecast(weather_forecast),
        format_availability(instructor_availability),
    )
}
```

**OpenAI API Integration**:
```rust
use async_openai::{Client, types::*};

pub struct RescheduleService {
    openai: Client,
    cache: Arc<redis::Client>,
}

impl RescheduleService {
    pub async fn generate_suggestions(
        &self,
        booking_id: &str,
        context: RescheduleContext,
    ) -> Result<Vec<RescheduleOption>> {
        // Check cache first
        let cache_key = format!("reschedule:{}:{}", booking_id, context.hash());
        if let Some(cached) = self.get_from_cache(&cache_key).await? {
            return Ok(cached);
        }

        // Build prompt
        let prompt = build_reschedule_prompt(
            &context.booking,
            &context.student,
            &context.weather_forecast,
            &context.instructor_availability,
        );

        // Call OpenAI
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4")
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content("You are a flight scheduling expert.")
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(prompt)
                    .build()?,
            ])
            .temperature(0.7)
            .build()?;

        let response = self.openai.chat().create(request).await?;

        let content = response.choices[0]
            .message
            .content
            .as_ref()
            .ok_or(AppError::InvalidAIResponse)?;

        // Parse response
        let suggestions: AISuggestionsResponse = serde_json::from_str(content)?;

        // Convert to RescheduleOptions
        let options = self.convert_to_options(suggestions, &context).await?;

        // Cache result
        self.save_to_cache(&cache_key, &options, 3600).await?;

        Ok(options)
    }

    async fn convert_to_options(
        &self,
        suggestions: AISuggestionsResponse,
        context: &RescheduleContext,
    ) -> Result<Vec<RescheduleOption>> {
        let mut options = Vec::new();

        for suggestion in suggestions.suggestions {
            // Check actual instructor availability
            let is_available = self.check_availability(
                &suggestion.datetime,
                context.instructor_id,
            ).await?;

            // Get weather forecast for that time
            let weather = self.get_weather_forecast(
                &context.booking.departure_location,
                &suggestion.datetime,
            ).await?;

            options.push(RescheduleOption {
                datetime: suggestion.datetime,
                reason: suggestion.reason,
                availability_status: if is_available {
                    AvailabilityStatus::Available
                } else {
                    AvailabilityStatus::Unavailable
                },
                weather_status: determine_weather_status(&weather),
                is_selectable: is_available,
            });
        }

        Ok(options)
    }
}
```

**Error Handling & Fallback**:
```rust
impl RescheduleService {
    pub async fn generate_suggestions_with_fallback(
        &self,
        booking_id: &str,
        context: RescheduleContext,
    ) -> Result<Vec<RescheduleOption>> {
        match self.generate_suggestions(booking_id, context.clone()).await {
            Ok(options) => Ok(options),
            Err(e) => {
                tracing::warn!("OpenAI API failed, using fallback: {}", e);
                self.generate_rule_based_suggestions(context).await
            }
        }
    }

    async fn generate_rule_based_suggestions(
        &self,
        context: RescheduleContext,
    ) -> Result<Vec<RescheduleOption>> {
        // Simple rule-based algorithm:
        // 1. Next available morning slot
        // 2. Next available afternoon slot
        // 3. Slot 2 days later (same time)

        // Implementation details...
        todo!()
    }
}
```

#### Dependencies
- Task 8 (Modal UI must exist to display suggestions)

#### PRD Alignment
✅ Implements **Feature 1: AI-Powered Reschedule System** (AI portion)
- FR-1.3: OpenAI Integration for Smart Suggestions

#### Estimated Effort
**3-4 weeks** (2 weeks per subtask)

---

### 🔴 Task 10: Implement Backend Reschedule API and Business Logic
**Complexity Score**: 7/10 | **Recommended Subtasks**: 2

#### Description
Develop the PATCH `/api/bookings/{id}/reschedule` endpoint with validation, updates, and notifications.

#### Why This Is Complex
- **Complex business logic**: Instructor availability, aircraft availability, student eligibility
- **Database transactions**: Atomicity required (update booking + create history record)
- **Notifications**: Email and WebSocket broadcasts
- **Integration**: Ties together weather, availability, and AI suggestions

#### Recommended Subtask Breakdown
1. **Endpoint Creation and Validation**
   - Create PATCH `/api/bookings/{id}/reschedule` endpoint
   - Validate new datetime (future, not conflicting)
   - Check instructor availability at new time
   - Check aircraft availability
   - Validate student eligibility for aircraft

2. **Update Logic and Notifications**
   - Update booking record in database
   - Create reschedule history record
   - Send WebSocket notification to student and instructor
   - Send email notifications (optional)
   - Handle rollback on failure

#### Implementation Details

**Request/Response Types**:
```rust
#[derive(Deserialize, Validate)]
pub struct RescheduleRequest {
    new_scheduled_date: DateTime<Utc>,
    reason: RescheduleReason,
    ai_suggestion_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum RescheduleReason {
    Weather,
    InstructorUnavailable,
    StudentRequest,
    AircraftMaintenance,
}

#[derive(Serialize)]
pub struct RescheduleResponse {
    booking: BookingResponse,
    reschedule_history_id: String,
}
```

**Endpoint Implementation**:
```rust
pub async fn reschedule_booking(
    Path(booking_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<RescheduleRequest>,
) -> Result<Json<RescheduleResponse>, AppError> {
    // Validate datetime is in future
    if req.new_scheduled_date <= Utc::now() {
        return Err(AppError::ValidationError(
            "New date must be in the future".to_string()
        ));
    }

    // Get existing booking
    let booking = get_booking(&state.db, &booking_id).await?
        .ok_or(AppError::NotFound)?;

    // Check instructor availability
    let instructor_available = check_instructor_availability(
        &state.db,
        booking.instructor_id,
        &req.new_scheduled_date,
        &booking.end_time,
    ).await?;

    if !instructor_available {
        return Err(AppError::ValidationError(
            "Instructor not available at requested time".to_string()
        ));
    }

    // Check aircraft availability
    let aircraft_available = check_aircraft_availability(
        &state.db,
        &booking.aircraft_type,
        &req.new_scheduled_date,
        &booking.end_time,
    ).await?;

    if !aircraft_available {
        return Err(AppError::ValidationError(
            "Aircraft not available at requested time".to_string()
        ));
    }

    // Begin transaction
    let mut tx = state.db.begin().await?;

    // Create reschedule history record
    let history_id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO booking_reschedule_history
         (id, booking_id, old_date, new_date, reason, ai_suggestion_id)
         VALUES (?, ?, ?, ?, ?, ?)",
        history_id,
        booking_id,
        booking.scheduled_date,
        req.new_scheduled_date,
        req.reason.to_string(),
        req.ai_suggestion_id,
    )
    .execute(&mut *tx)
    .await?;

    // Update booking
    let updated_booking = sqlx::query_as!(
        Booking,
        "UPDATE bookings
         SET scheduled_date = ?, status = 'Rescheduled'
         WHERE id = ?
         RETURNING *",
        req.new_scheduled_date,
        booking_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    // Send notifications (async, don't wait)
    tokio::spawn(send_reschedule_notifications(
        state.clone(),
        updated_booking.clone(),
        booking.scheduled_date,
    ));

    Ok(Json(RescheduleResponse {
        booking: updated_booking.into(),
        reschedule_history_id: history_id,
    }))
}
```

**Availability Checks**:
```rust
async fn check_instructor_availability(
    db: &SqlitePool,
    instructor_id: i64,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<bool> {
    let conflicts = sqlx::query!(
        "SELECT COUNT(*) as count FROM bookings
         WHERE instructor_id = ?
         AND status NOT IN ('Cancelled', 'Completed')
         AND (
             (scheduled_date >= ? AND scheduled_date < ?) OR
             (end_time > ? AND end_time <= ?) OR
             (scheduled_date <= ? AND end_time >= ?)
         )",
        instructor_id,
        start_time,
        end_time,
        start_time,
        end_time,
        start_time,
        end_time,
    )
    .fetch_one(db)
    .await?;

    Ok(conflicts.count == 0)
}

async fn check_aircraft_availability(
    db: &SqlitePool,
    aircraft_type: &str,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<bool> {
    // Similar logic to instructor availability
    // Check for overlapping bookings with same aircraft
    todo!()
}
```

**Notifications**:
```rust
async fn send_reschedule_notifications(
    state: AppState,
    booking: Booking,
    old_date: DateTime<Utc>,
) {
    // WebSocket notification
    let notification = json!({
        "type": "booking_rescheduled",
        "booking_id": booking.id,
        "old_date": old_date,
        "new_date": booking.scheduled_date,
    });

    if let Err(e) = state.ws_broadcaster.send_to_user(
        booking.student_id,
        notification.to_string(),
    ).await {
        tracing::error!("Failed to send WebSocket notification: {}", e);
    }

    // Email notification (optional)
    if let Err(e) = send_email_notification(&booking, &old_date).await {
        tracing::error!("Failed to send email notification: {}", e);
    }
}
```

#### Dependencies
- Task 9 (AI suggestions must be available to track which suggestion was used)

#### PRD Alignment
✅ Implements **Feature 1: AI-Powered Reschedule System** (backend portion)
- FR-1.5: Backend API Endpoint
- FR-1.7: Success Feedback

#### Estimated Effort
**3-4 weeks** (2 weeks per subtask)

---

## Dependency Graph

```
Task 1 (WebSocket) ─┬─→ Task 2 (Status Indicator)
                    └─→ Task 6 (Alert Banner) ──→ Task 7 (Weather Service)

Task 3 (Error Handling) ─┬─→ Task 4 (Loading States) ──┐
                         └─→ Task 5 (Validation)        │
                                                          ├─→ Task 8 (Modal UI) ──→ Task 9 (OpenAI) ──→ Task 10 (Reschedule API)
```

**Critical Path** (longest sequential chain):
Task 3 → Task 4 → Task 8 → Task 9 → Task 10
**Estimated Duration**: 13-17 weeks

**Parallel Tracks**:
- Track A: WebSocket & Weather (Tasks 1, 2, 6, 7) - ~6 weeks
- Track B: Reschedule System (Tasks 3, 4, 5, 8, 9, 10) - ~13-17 weeks

---

## Recommended Implementation Order

### Phase 1: Foundation (Weeks 1-5)
**Parallel Work**:
- **Track A**: Task 1 (WebSocket) + Task 2 (Status Indicator)
- **Track B**: Task 3 (Error Handling) + Task 5 (Validation)

**Rationale**: Build the foundational infrastructure that other features depend on

### Phase 2: User Feedback (Weeks 5-7)
- Task 4 (Loading States)

**Rationale**: With error handling in place, add comprehensive user feedback

### Phase 3: Weather System (Weeks 6-9)
**Parallel Work**:
- Task 6 (Alert Banner)
- Task 7 (Weather Service)

**Rationale**: Can work in parallel with Phase 2. Depends on WebSocket from Phase 1

### Phase 4: Reschedule System (Weeks 7-14)
**Sequential Work**:
- Task 8 (Modal UI) - Weeks 7-9
- Task 9 (OpenAI Integration) - Weeks 9-13
- Task 10 (Reschedule API) - Weeks 11-14

**Rationale**: Sequential due to dependencies, but can overlap Task 9 and 10 slightly

### Phase 5: Testing & Refinement (Weeks 14-16)
- E2E test suite completion
- Performance testing
- Bug fixes
- Documentation

---

## Risk Assessment

### High-Risk Items

1. **Task 9: OpenAI API Integration**
   - **Risk**: API costs could exceed budget
   - **Mitigation**: Aggressive caching (1 hour TTL), usage limits, fallback to rule-based

2. **Task 1: WebSocket Infrastructure**
   - **Risk**: Complex reconnection logic could have edge cases
   - **Mitigation**: Extensive testing, phased rollout, monitoring

3. **Task 10: Reschedule API**
   - **Risk**: Race conditions in availability checking
   - **Mitigation**: Database transactions, row-level locking, conflict detection

### Medium-Risk Items

1. **Task 7: Weather Monitoring Service**
   - **Risk**: External API rate limits or downtime
   - **Mitigation**: Caching, fallback to last known data, manual override

2. **Task 5: Form Validation**
   - **Risk**: Client/server validation mismatch
   - **Mitigation**: Shared validation rules, comprehensive testing

---

## Resource Requirements

### Development Team
- **2 Backend Engineers**: Rust/Axum/WebSocket expertise
- **1 Frontend Engineer**: Elm expertise
- **1 Full-Stack Engineer**: Can work on both sides
- **1 QA Engineer**: Testing and automation

### External Services
- **OpenAI API**: GPT-4 access ($$$)
- **OpenWeatherMap API**: Pro plan for frequent polling
- **Redis**: For caching and rate limiting
- **Email Service**: (Optional) For notifications

### Infrastructure
- WebSocket-capable hosting (AWS ALB, or similar)
- Horizontal scaling capability for WebSocket connections
- Database backups and replication

---

## Success Criteria

### Technical Metrics
- [ ] All 10 tasks completed and passing E2E tests
- [ ] WebSocket uptime > 99.5%
- [ ] API error rate < 1%
- [ ] OpenAI API response time < 3 seconds
- [ ] Weather check frequency: every 5 minutes
- [ ] Zero data loss during reconnection

### User Experience Metrics
- [ ] Reschedule task completion > 95%
- [ ] Error recovery rate > 90%
- [ ] User satisfaction with AI suggestions > 80%

### Business Metrics
- [ ] Weather-related cancellations reduced by 30%
- [ ] Support tickets for errors reduced by 50%
- [ ] Average time to reschedule < 2 minutes

---

## Alignment with PRD

This task list perfectly aligns with the features defined in [prd-next.md](./prd-next.md):

| PRD Feature | Tasks Covering It |
|-------------|-------------------|
| Feature 1: AI-Powered Reschedule System | Tasks 8, 9, 10 |
| Feature 2: Real-Time Weather Alert System | Tasks 6, 7 |
| Feature 3: Enhanced WebSocket Infrastructure | Tasks 1, 2 |
| Feature 4: Comprehensive Error Handling | Task 3 |
| Feature 5: Loading States & User Feedback | Task 4 |
| Feature 6: Form Validation Enhancements | Task 5 |

**Coverage**: 100% ✅

---

## Next Steps

1. **Review this document** with the team
2. **Prioritize tasks** based on business value
3. **Assign tasks** to engineers based on expertise
4. **Set up project tracking** (Jira, Linear, etc.)
5. **Kick off Phase 1** with Tasks 1 and 3 in parallel
6. **Schedule weekly sync** to track progress
7. **Set up monitoring** for success metrics

---

## Appendix: Complexity Justifications

### Why Task 1 is Score 8 (Highest)
- Cross-language coordination (Elm + Rust)
- Async programming with tokio
- State management across reconnections
- Edge cases: concurrent disconnects, message ordering
- Testing complexity: network simulation, timing issues

### Why Tasks 7, 9, 10 are Score 7
- External API integration (reliability, error handling)
- Business logic complexity (weather analysis, availability checking)
- Data consistency requirements (transactions, rollback)
- Notification broadcasting (fan-out to multiple clients)

### Why Tasks 3, 5 are Score 6
- Dual implementation (client + server)
- Middleware integration (Redis)
- Extensive testing required
- Coordination between frontend and backend

### Why Tasks 4, 6, 8 are Score 5
- Primarily UI work
- Well-defined requirements
- Standard patterns
- Limited external dependencies

### Why Task 2 is Score 4 (Lowest)
- Purely UI component
- Simple reactive updates
- No complex logic
- Depends on Task 1 but otherwise straightforward

---

**Document Status**: Ready for Review
**Last Updated**: 2025-11-08
**Next Review**: After Phase 1 completion
