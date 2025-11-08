#!/bin/bash

# E2E Test Runner Script
# This script sets up the environment and runs E2E tests

set -e

echo "🚀 Setting up E2E test environment..."

# Check if we're in the right directory
if [ ! -d "e2e" ]; then
    echo "❌ Error: e2e directory not found. Run this script from the project root."
    exit 1
fi

# Set test environment variables
export DATABASE_URL="sqlite:/tmp/weather_app_test.db"
export WEATHER_API_KEY="test_weather_key"
export WEATHER_API_BASE_URL="https://api.openweathermap.org/data/2.5"
export OPENAI_API_KEY="test_openai_key"
export RESEND_API_KEY="test_resend_key"
export FROM_EMAIL="test@flightschedulepro.com"
export RUST_LOG="error"

# Copy test env file to root so server can read it
cp e2e/.env.test .env

echo "📦 Installing E2E dependencies..."
cd e2e
pnpm install

echo "🌐 Installing Playwright browsers..."
pnpx playwright install chromium

echo "🔨 Building backend..."
cd ../server
cargo build

echo "🎨 Installing frontend dependencies..."
cd ../elm
pnpm install

echo "🧪 Running E2E tests..."
cd ../e2e
pnpm run test

echo "✅ E2E tests completed successfully!"