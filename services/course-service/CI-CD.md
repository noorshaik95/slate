# CI/CD Pipeline Documentation

This document describes the continuous integration and deployment pipeline for the Course Management Service.

## Pipeline Overview

The CI/CD pipeline is implemented using GitHub Actions and consists of multiple jobs that run on every push and pull request.

## Workflow: `course-service-ci.yml`

### Triggers

The workflow is triggered on:
- **Push** to branches: `main`, `develop`, `claude/**`
- **Pull requests** to: `main`, `develop`
- Only when changes affect:
  - `services/course-service/**`
  - `proto/course.proto`
  - `.github/workflows/course-service-ci.yml`

### Jobs

#### 1. Test Job

**Purpose**: Run unit tests and integration tests with coverage reporting

**Matrix Strategy**: Tests run on Node.js 18.x and 20.x

**Services**:
- MongoDB 7 (for integration tests)

**Steps**:
1. Checkout code
2. Setup Node.js with npm caching
3. Install dependencies
4. Run linter
5. Run unit tests with coverage
6. Check coverage threshold (80% minimum)
7. Upload coverage to Codecov
8. Run integration tests
9. Archive test results

**Coverage Requirements**:
- Lines: ≥ 80%
- Functions: ≥ 80%
- Branches: ≥ 75%
- Statements: ≥ 80%

**Artifacts**:
- Test results
- Coverage reports

#### 2. Lint Job

**Purpose**: Ensure code quality and formatting standards

**Steps**:
1. Checkout code
2. Setup Node.js
3. Install dependencies
4. Run ESLint
5. Check code formatting with Prettier

**Configuration**:
- ESLint rules: `.eslintrc.js`
- Prettier config: `.prettierrc`

#### 3. Build Job

**Purpose**: Compile TypeScript and verify build succeeds

**Dependencies**: Requires `test` and `lint` jobs to pass

**Steps**:
1. Checkout code
2. Setup Node.js
3. Install dependencies
4. Build application (`npm run build`)
5. Archive build artifacts

**Artifacts**:
- `dist/` directory (retained for 7 days)

#### 4. Docker Job

**Purpose**: Build and test Docker image

**Dependencies**: Requires `build` job to pass

**Conditions**: Only runs on push to `main` or `develop` branches

**Steps**:
1. Checkout code
2. Setup Docker Buildx
3. Build Docker image with caching
4. Test Docker image

**Tags**:
- `course-service:latest`
- `course-service:<commit-sha>`

**Caching**: Uses GitHub Actions cache for faster builds

#### 5. Security Job

**Purpose**: Scan for security vulnerabilities

**Steps**:
1. Run `npm audit` (moderate level and above)
2. Run Snyk security scan (high severity threshold)

**Note**: Security job failures are non-blocking (`continue-on-error: true`)

## Running Locally

### Run All Tests
```bash
cd services/course-service
npm test
```

### Run Tests with Coverage
```bash
npm run test:cov
```

### Run Integration Tests
```bash
# Start MongoDB
docker run -d -p 27017:27017 mongo:7-jammy

# Run tests
npm run test:e2e
```

### Run Linter
```bash
npm run lint
```

### Fix Lint Issues
```bash
npm run lint:fix
```

### Check Formatting
```bash
npm run format:check
```

### Fix Formatting
```bash
npm run format
```

### Build
```bash
npm run build
```

## Coverage Reports

Coverage reports are generated in the `coverage/` directory:

- `coverage/lcov-report/index.html` - HTML report (open in browser)
- `coverage/coverage-summary.json` - JSON summary
- `coverage/lcov.info` - LCOV format (for CI tools)

### Viewing Coverage Locally

```bash
npm run test:cov
open coverage/lcov-report/index.html
```

## Test Structure

### Unit Tests

Located in `src/**/*.spec.ts`

**Coverage**:
- CourseService: 139 test cases
- EnrollmentService: 15 test cases
- CourseRepository: 9 test cases
- EnrollmentRepository: 7 test cases
- CourseController: 10 test cases
- MetricsInterceptor: 3 test cases
- HealthController: 6 test cases
- MetricsService: 10 test cases

**Total**: 199 unit test cases

### Integration Tests

Located in `test/course.e2e-spec.ts`

**Test Suites**:
1. Course Creation and Management (8 tests)
2. Enrollment Workflow (6 tests)
3. Course Templates (2 tests)
4. Prerequisites (4 tests)
5. Co-teaching (2 tests)
6. Sections (4 tests)

**Total**: 26 integration test cases

## CI/CD Best Practices

### 1. Fast Feedback

- Tests run in parallel across Node.js versions
- Linting runs independently
- Early failure stops dependent jobs

### 2. Caching

- npm packages cached per Node.js version
- Docker layers cached with GitHub Actions cache
- Reduces build time by ~50%

### 3. Artifact Management

- Test results retained for debugging
- Build artifacts retained for 7 days
- Coverage reports uploaded to Codecov

### 4. Security

- Automated vulnerability scanning
- Audit on every push
- Non-blocking to avoid false positives

### 5. Quality Gates

- 80% code coverage required
- All tests must pass
- No linting errors allowed
- Build must succeed

## Environment Variables

### Required for CI

None - all dependencies are self-contained

### Optional

- `SNYK_TOKEN`: For Snyk security scanning
- `CODECOV_TOKEN`: For Codecov uploads (optional, works without)

## Troubleshooting

### Tests Failing Locally but Passing in CI

- Ensure you're using the same Node.js version (18.x or 20.x)
- Clear `node_modules` and reinstall: `rm -rf node_modules && npm ci`
- Check MongoDB version matches (7.x)

### Coverage Below Threshold

- Run `npm run test:cov` to see which files need more tests
- Check `coverage/lcov-report/index.html` for detailed breakdown
- Add tests for uncovered lines/branches

### Build Failures

- Check TypeScript errors: `npm run build`
- Verify all imports are correct
- Check for missing dependencies in `package.json`

### Linting Failures

- Run `npm run lint:fix` to auto-fix issues
- Run `npm run format` to fix formatting
- Check `.eslintrc.js` for rule configurations

### Docker Build Failures

- Test locally: `docker build -f services/course-service/Dockerfile .`
- Check Dockerfile syntax
- Verify all files are copied correctly

## Future Enhancements

- [ ] Add performance testing
- [ ] Add E2E tests with full stack
- [ ] Implement automatic deployment to staging
- [ ] Add mutation testing
- [ ] Integrate with SonarQube for code quality
- [ ] Add contract testing for gRPC
- [ ] Implement canary deployments
- [ ] Add load testing in CI

## Badges

Add these to your README:

```markdown
![Tests](https://github.com/your-org/slate/workflows/Course%20Service%20CI%2FCD/badge.svg)
[![codecov](https://codecov.io/gh/your-org/slate/branch/main/graph/badge.svg)](https://codecov.io/gh/your-org/slate)
```

## Monitoring

### Pipeline Health

Monitor these metrics:
- Build success rate
- Average build time
- Test failure rate
- Coverage trends

### Alerts

Set up alerts for:
- Build failures on `main` branch
- Coverage drops below 75%
- Security vulnerabilities (high/critical)
- Build time exceeds 10 minutes

## Support

For CI/CD issues:
1. Check GitHub Actions logs
2. Review this documentation
3. Check test output locally
4. Consult the team lead

---

**Last Updated**: 2025-01-15
**Maintained By**: DevOps Team
