-- Add aircraft_type column to bookings table
ALTER TABLE bookings ADD COLUMN aircraft_type TEXT NOT NULL DEFAULT 'Cessna 172';