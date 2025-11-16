# Testing Guide - Course Management Service

This document provides information about the test suite for the Course Management Service.

## Test Coverage

The service includes comprehensive unit tests for:

### Service Layer
- **CourseService** (139 tests)
  - Course CRUD operations
  - Publishing/unpublishing
  - Template operations
  - Prerequisites with circular dependency detection
  - Co-instructor management
  - Section management
  - Cross-listing operations

- **EnrollmentService** (15 tests)
  - Self-enrollment with validation
  - Instructor-add enrollment
  - Enrollment removal
  - Course roster management
  - Student enrollment history

### Repository Layer
- **CourseRepository** (9 tests)
  - Database operations
  - Query filtering
  - Update operations
  - Prerequisite/co-instructor management

- **EnrollmentRepository** (7 tests)
  - Enrollment queries
  - Filtering by course/student/section/status
  - Counting operations

### Controller Layer
- **CourseController** (10 tests)
  - gRPC endpoint handling
  - Request/response transformation
  - Proto message formatting

### Interceptors
- **MetricsInterceptor** (3 tests)
  - Request duration tracking
  - Success/failure metrics
  - Timing accuracy

## Running Tests

### Run All Tests
```bash
npm test
```

### Run Tests in Watch Mode
```bash
npm run test:watch
```

### Run Tests with Coverage
```bash
npm run test:cov
```

This generates a coverage report in the `coverage/` directory.

### Run E2E Tests
```bash
npm run test:e2e
```

### Run Specific Test File
```bash
npm test -- course.service.spec.ts
```

### Run Tests Matching Pattern
```bash
npm test -- --testNamePattern="should create a course"
```

## Test Structure

Tests follow the AAA pattern (Arrange, Act, Assert):

```typescript
it('should create a course successfully', async () => {
  // Arrange
  courseRepository.create.mockResolvedValue(mockCourse as any);

  // Act
  const result = await service.createCourse({
    title: 'Test Course',
    // ...
  });

  // Assert
  expect(courseRepository.create).toHaveBeenCalled();
  expect(metricsService.incrementCoursesCreated).toHaveBeenCalledWith(true);
  expect(result).toEqual(mockCourse);
});
```

## Test Utilities

The `test/test-utils.ts` file provides helper functions:

- `createMockCourse()` - Generate mock course objects
- `createMockEnrollment()` - Generate mock enrollment objects
- `createMockTemplate()` - Generate mock template objects
- `createMockSection()` - Generate mock section objects
- `createMockMongooseModel()` - Create Mongoose model mocks
- `createMockExecutionContext()` - Create NestJS execution context mocks

Example usage:

```typescript
import { createMockCourse } from '../test/test-utils';

const course = createMockCourse({
  title: 'Custom Title',
  isPublished: true,
});
```

## Mocking Strategy

### Service Mocks
Services are mocked using Jest's `jest.fn()`:

```typescript
const mockCourseService = {
  createCourse: jest.fn(),
  getCourse: jest.fn(),
  // ...
};
```

### Repository Mocks
Repositories use mocked Mongoose models:

```typescript
const mockModel = {
  find: jest.fn(),
  findById: jest.fn(),
  exec: jest.fn(),
};
```

### Metrics Mocks
Metrics service is mocked to verify tracking:

```typescript
const mockMetricsService = {
  incrementCoursesCreated: jest.fn(),
  observeGrpcRequestDuration: jest.fn(),
};
```

## Coverage Goals

Target coverage metrics:
- **Statements**: > 80%
- **Branches**: > 75%
- **Functions**: > 80%
- **Lines**: > 80%

## Best Practices

1. **Test Isolation**: Each test should be independent
2. **Clear Names**: Use descriptive test names (should ...)
3. **Single Assertion**: Test one behavior per test
4. **Mock External Dependencies**: Don't hit real databases
5. **Clean Up**: Use `afterEach` to clear mocks

## Continuous Integration

Tests are run automatically on:
- Pull request creation
- Push to main branch
- Before deployment

## Common Test Scenarios

### Testing Error Cases
```typescript
it('should throw NotFoundException when course not found', async () => {
  courseRepository.findById.mockResolvedValue(null);

  await expect(service.getCourse('non-existent-id'))
    .rejects.toThrow(NotFoundException);
});
```

### Testing Metrics Tracking
```typescript
it('should track failed course creation in metrics', async () => {
  courseRepository.create.mockRejectedValue(new Error('DB error'));

  await expect(service.createCourse(data)).rejects.toThrow();

  expect(metricsService.incrementCoursesCreated)
    .toHaveBeenCalledWith(false);
});
```

### Testing Business Logic
```typescript
it('should detect circular prerequisite dependencies', async () => {
  // Setup circular dependency
  courseRepository.findById
    .mockResolvedValueOnce(course1)
    .mockResolvedValueOnce(course2);

  await expect(service.addPrerequisite('course-1', 'course-2'))
    .rejects.toThrow('Circular prerequisite dependency detected');
});
```

## Debugging Tests

### Run Single Test with Debug Output
```bash
npm test -- --verbose course.service.spec.ts
```

### Debug in VS Code
Add to `.vscode/launch.json`:
```json
{
  "type": "node",
  "request": "launch",
  "name": "Jest Debug",
  "program": "${workspaceFolder}/node_modules/.bin/jest",
  "args": ["--runInBand", "--no-cache"],
  "console": "integratedTerminal"
}
```

## Future Enhancements

- [ ] Add integration tests with test database
- [ ] Add performance/load tests
- [ ] Add contract tests for gRPC endpoints
- [ ] Increase coverage to > 90%
- [ ] Add mutation testing
