-- Add weather alerts table for persistent alert storage
CREATE TABLE IF NOT EXISTS weather_alerts (
    id TEXT PRIMARY KEY NOT NULL,
    booking_id TEXT,
    severity TEXT NOT NULL CHECK (
        severity IN ('severe', 'high', 'moderate', 'low', 'clear')
    ),
    message TEXT NOT NULL,
    location TEXT NOT NULL,
    student_name TEXT,
    original_date TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dismissed_at TIMESTAMP,
    FOREIGN KEY (booking_id) REFERENCES bookings(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_weather_alerts_severity ON weather_alerts(severity);
CREATE INDEX IF NOT EXISTS idx_weather_alerts_created_at ON weather_alerts(created_at);
CREATE INDEX IF NOT EXISTS idx_weather_alerts_booking_id ON weather_alerts(booking_id);
CREATE INDEX IF NOT EXISTS idx_weather_alerts_dismissed_at ON weather_alerts(dismissed_at);
