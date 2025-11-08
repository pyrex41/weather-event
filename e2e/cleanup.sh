#!/bin/bash

# E2E Test Environment Cleanup Script

echo "🧹 Cleaning up E2E test environment..."

# Kill processes using test ports
echo "Killing processes on ports 3000 (backend) and 5173 (frontend)..."
lsof -ti:3000 | xargs kill -9 2>/dev/null || echo "No process found on port 3000"
lsof -ti:5173 | xargs kill -9 2>/dev/null || echo "No process found on port 5173"

# Remove test databases
echo "Removing test database files..."
rm -f weather_app_test.db
rm -f *.db

# Clean up Playwright artifacts
echo "Cleaning up Playwright artifacts..."
cd e2e
rm -rf test-results/
rm -rf playwright-report/
cd ..

echo "✅ Cleanup completed!"