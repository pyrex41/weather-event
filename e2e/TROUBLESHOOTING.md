# E2E Testing Troubleshooting Guide

## Common Issues and Solutions

### Tests Failing Due to Port Conflicts

**Problem:** `Error: listen EADDRINUSE: address already in use`

**Solution:**
```bash
# Kill processes using ports 3000 and 5173
lsof -ti:3000 | xargs kill -9
lsof -ti:5173 | xargs kill -9

# Or use the cleanup script
./e2e/cleanup.sh
```

### WebSocket Connection Issues

**Problem:** WebSocket tests failing with connection errors

**Solution:**
- Ensure backend is running on port 3000
- Check CORS configuration allows localhost:5173
- Verify WebSocket endpoint `/ws` is accessible

### API Mocking Not Working

**Problem:** Tests making real API calls instead of using mocks

**Solution:**
- Verify `apiMocks.setupAllMocks()` is called in `test.beforeEach`
- Check that mock routes are set up before page navigation
- Ensure mock URLs match the exact API endpoints

### Browser Launch Failures

**Problem:** `browserType.launch: Executable doesn't exist`

**Solution:**
```bash
cd e2e
npx playwright install chromium
# or for all browsers
npx playwright install
```

### Slow Test Execution

**Problem:** Tests taking longer than 30 seconds

**Solution:**
- Verify all external APIs are properly mocked
- Check for unnecessary `page.waitForTimeout()` calls
- Ensure database is using in-memory SQLite for tests
- Run tests in parallel with `workers: undefined` in config

### Element Not Found Errors

**Problem:** `locator.waitFor` timeouts

**Solution:**
- Verify data-testid attributes match test selectors
- Check that page has fully loaded before interactions
- Add explicit waits for async operations
- Use `page.waitForLoadState('networkidle')` if needed

### CI/CD Pipeline Issues

**Problem:** Tests pass locally but fail in CI

**Solution:**
- Ensure CI environment has Node.js 20+ and Rust 1.75+
- Verify all system dependencies are installed (SQLite, etc.)
- Check that test database is properly initialized
- Review CI logs for specific error messages

### Debugging Tips

```bash
# Run tests with debug mode
cd e2e
npm run test:debug

# Run specific test file
npx playwright test booking-creation.spec.ts

# Run with headed browser to see what's happening
npm run test:headed

# Generate and view HTML report
npm run report

# Check test configuration
npx playwright test --list
```

### Performance Optimization

- Keep mocks lightweight and fast
- Use `page.route()` for API interception instead of real calls
- Minimize browser context switching
- Run tests in parallel when possible
- Use appropriate wait strategies (avoid arbitrary timeouts)

### Environment Variables

Ensure these are set for tests:
```env
DATABASE_URL=sqlite::memory:
WEATHER_API_KEY=test_weather_key
OPENAI_API_KEY=test_openai_key
RESEND_API_KEY=test_resend_key
RUST_LOG=error
```