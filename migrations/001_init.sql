-- Initial database schema for weather event cancellation system

-- Students table
CREATE TABLE IF NOT EXISTS students (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    training_level TEXT NOT NULL CHECK (
        training_level IN ('STUDENT_PILOT', 'PRIVATE_PILOT', 'INSTRUMENT_RATED')
    ),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_students_email ON students(email);
CREATE INDEX idx_students_training_level ON students(training_level);

-- Bookings table
CREATE TABLE IF NOT EXISTS bookings (
    id TEXT PRIMARY KEY NOT NULL,
    student_id TEXT NOT NULL,
    scheduled_date TIMESTAMP NOT NULL,
    departure_location TEXT NOT NULL, -- JSON: {"lat": float, "lon": float, "name": string}
    status TEXT NOT NULL CHECK (
        status IN ('SCHEDULED', 'CANCELLED', 'RESCHEDULED', 'COMPLETED')
    ),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE CASCADE
);

CREATE INDEX idx_bookings_student_id ON bookings(student_id);
CREATE INDEX idx_bookings_scheduled_date ON bookings(scheduled_date);
CREATE INDEX idx_bookings_status ON bookings(status);
CREATE INDEX idx_bookings_status_date ON bookings(status, scheduled_date);

-- Weather checks table
CREATE TABLE IF NOT EXISTS weather_checks (
    id TEXT PRIMARY KEY NOT NULL,
    booking_id TEXT NOT NULL,
    checked_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    weather_data TEXT NOT NULL, -- JSON weather data
    is_safe BOOLEAN NOT NULL,
    reason TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (booking_id) REFERENCES bookings(id) ON DELETE CASCADE
);

CREATE INDEX idx_weather_checks_booking_id ON weather_checks(booking_id);
CREATE INDEX idx_weather_checks_checked_at ON weather_checks(checked_at);
CREATE INDEX idx_weather_checks_is_safe ON weather_checks(is_safe);

-- Reschedule events table
CREATE TABLE IF NOT EXISTS reschedule_events (
    id TEXT PRIMARY KEY NOT NULL,
    booking_id TEXT NOT NULL,
    original_date TIMESTAMP NOT NULL,
    new_date TIMESTAMP NOT NULL,
    suggested_by TEXT NOT NULL, -- 'AI', 'STUDENT', 'INSTRUCTOR'
    ai_suggestions TEXT, -- JSON array of AI-generated options
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (booking_id) REFERENCES bookings(id) ON DELETE CASCADE
);

CREATE INDEX idx_reschedule_booking_id ON reschedule_events(booking_id);
CREATE INDEX idx_reschedule_original_date ON reschedule_events(original_date);
CREATE INDEX idx_reschedule_suggested_by ON reschedule_events(suggested_by);

-- Weather minimums table (configurable per training level)
CREATE TABLE IF NOT EXISTS weather_minimums (
    id TEXT PRIMARY KEY NOT NULL,
    training_level TEXT NOT NULL UNIQUE CHECK (
        training_level IN ('STUDENT_PILOT', 'PRIVATE_PILOT', 'INSTRUMENT_RATED')
    ),
    min_visibility_sm REAL NOT NULL,
    max_wind_speed_kt REAL NOT NULL,
    min_ceiling_ft REAL,
    allow_imc BOOLEAN NOT NULL DEFAULT 0,
    no_thunderstorms BOOLEAN NOT NULL DEFAULT 1,
    no_icing BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_weather_minimums_training_level ON weather_minimums(training_level);

-- Insert default weather minimums
INSERT OR IGNORE INTO weather_minimums (id, training_level, min_visibility_sm, max_wind_speed_kt, min_ceiling_ft, allow_imc, no_thunderstorms, no_icing)
VALUES
    ('default_student', 'STUDENT_PILOT', 5.0, 12.0, 3000.0, 0, 1, 1),
    ('default_private', 'PRIVATE_PILOT', 3.0, 20.0, 1000.0, 0, 1, 1),
    ('default_instrument', 'INSTRUMENT_RATED', 1.0, 30.0, NULL, 1, 1, 1);

-- Enable WAL mode for better concurrent access
PRAGMA journal_mode=WAL;

-- Enable foreign keys
PRAGMA foreign_keys=ON;
