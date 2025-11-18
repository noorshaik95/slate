# gRPC to REST API Naming Convention

This document defines the naming conventions for gRPC methods that enable automatic mapping to RESTful HTTP routes in the API Gateway.

## Overview

The API Gateway uses **method name patterns** to automatically generate REST API routes from gRPC service definitions. This eliminates the need for manual route configuration while maintaining RESTful conventions.

## Naming Convention Rules

### Tier 1: Simple Resources (Top-level CRUD)

For operations on a single, top-level resource:

| gRPC Method Pattern | HTTP Method | HTTP Path | Description |
|-------------------|-------------|-----------|-------------|
| `Get{Resource}` | GET | `/api/{resources}/:id` | Retrieve a single resource by ID |
| `List{Resources}` | GET | `/api/{resources}` | Retrieve a list of resources |
| `Create{Resource}` | POST | `/api/{resources}` | Create a new resource |
| `Update{Resource}` | PUT | `/api/{resources}/:id` | Update an existing resource |
| `Delete{Resource}` | DELETE | `/api/{resources}/:id` | Delete a resource |

**Examples:**
```protobuf
rpc GetUser(GetUserRequest) returns (UserResponse);
// Maps to: GET /api/users/:id

rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
// Maps to: GET /api/users

rpc CreateUser(CreateUserRequest) returns (UserResponse);
// Maps to: POST /api/users

rpc UpdateUser(UpdateUserRequest) returns (UserResponse);
// Maps to: PUT /api/users/:id

rpc DeleteUser(DeleteUserRequest) returns (google.protobuf.Empty);
// Maps to: DELETE /api/users/:id
```

### Tier 2: Nested Resources (Sub-resources)

For operations on resources that belong to a parent resource:

#### Pattern: `{Action}{ParentResource}{ChildResource}`

**Collection Operations:**

| gRPC Method Pattern | HTTP Method | HTTP Path | Description |
|-------------------|-------------|-----------|-------------|
| `List{Parent}{Children}` | GET | `/api/{parents}/:id/{children}` | List all child resources of a parent |
| `Get{Parent}{Children}` | GET | `/api/{parents}/:id/{children}` | Alternative pattern for listing children |

**Member Operations:**

| gRPC Method Pattern | HTTP Method | HTTP Path | Description |
|-------------------|-------------|-----------|-------------|
| `Get{Parent}{Child}` | GET | `/api/{parents}/:id/{children}/:child_id` | Get a specific child resource |
| `Add{Parent}{Child}` | POST | `/api/{parents}/:id/{children}` | Add a new child resource to parent |
| `Update{Parent}{Child}` | PUT | `/api/{parents}/:id/{children}/:child_id` | Update a child resource |
| `Remove{Parent}{Child}` | DELETE | `/api/{parents}/:id/{children}/:child_id` | Remove a child from parent |

**Examples:**
```protobuf
// Group Members (nested under Groups)
rpc AddGroupMember(AddGroupMemberRequest) returns (google.protobuf.Empty);
// Maps to: POST /api/groups/:id/members

rpc RemoveGroupMember(RemoveGroupMemberRequest) returns (google.protobuf.Empty);
// Maps to: DELETE /api/groups/:id/members/:user_id

rpc GetGroupMembers(GetGroupMembersRequest) returns (GetGroupMembersResponse);
// Maps to: GET /api/groups/:id/members

rpc ListGroupMembers(ListGroupMembersRequest) returns (ListGroupMembersResponse);
// Maps to: GET /api/groups/:id/members

// User Groups (nested under Users)
rpc GetUserGroups(GetUserGroupsRequest) returns (GetUserGroupsResponse);
// Maps to: GET /api/users/:id/groups

// Course Assignments (nested under Courses)
rpc AddCourseAssignment(AddCourseAssignmentRequest) returns (AssignmentResponse);
// Maps to: POST /api/courses/:id/assignments

rpc RemoveCourseAssignment(RemoveCourseAssignmentRequest) returns (google.protobuf.Empty);
// Maps to: DELETE /api/courses/:id/assignments/:assignment_id
```

## Resource Name Guidelines

### Singular vs Plural

- **Operation prefixes** use singular form: `Get`, `Create`, `Update`, `Delete`, `Add`, `Remove`
- **List operation** uses plural form: `List{Resources}`
- **HTTP paths** always use plural form: `/api/users`, `/api/groups/members`

### CamelCase Parsing

The system automatically parses CamelCase resource names:

- `User` → `users`
- `Group` → `groups`
- `GroupMember` → Split into `Group` (parent) + `Member` (child)
- `CourseAssignment` → Split into `Course` (parent) + `Assignment` (child)

### Pluralization Rules

The system applies English pluralization rules:

- Most words: add `s` → `user` → `users`
- Words ending in `s`, `ss`, `sh`, `ch`, `x`, `z`: add `es` → `class` → `classes`
- Words ending in consonant + `y`: change to `ies` → `category` → `categories`
- Words ending in vowel + `y`: add `s` → `day` → `days`

## Path Parameter Conventions

### Simple Resources
- Primary ID parameter: `:id`
  - Example: `/api/users/:id`

### Nested Resources
- Parent ID parameter: `:id`
- Child ID parameter: Named after the child resource
  - `:user_id`, `:member_id`, `:assignment_id`, etc.

**Examples:**
```
DELETE /api/groups/:id/members/:user_id
GET /api/courses/:id/assignments/:assignment_id
PUT /api/users/:id/permissions/:permission_id
```

## Operation Prefix Guide

### Standard CRUD Operations
- `Get` - Retrieve a single resource
- `List` - Retrieve multiple resources
- `Create` - Create a new resource
- `Update` - Modify an existing resource
- `Delete` - Remove a resource

### Nested Resource Operations
- `Add` - Add a child resource to a parent (POST)
- `Remove` - Remove a child resource from a parent (DELETE)
- `Get{Parent}{Children}` or `List{Parent}{Children}` - List children (GET)
- `Get{Parent}{Child}` - Get a specific child (GET)
- `Update{Parent}{Child}` - Update a specific child (PUT)

## Special Cases and Exceptions

### Non-CRUD Operations

Methods that don't follow CRUD patterns (e.g., `Login`, `Register`, `AssignRole`) will **not** be automatically mapped and should be:
1. Manually configured using route overrides, OR
2. Renamed to follow the convention where possible

### Complex Nested Resources

For deeply nested resources (3+ levels), consider:
1. Flattening the hierarchy where possible
2. Using manual route overrides for specific cases
3. Restructuring the API to avoid deep nesting

### Custom Actions

For custom actions that don't fit CRUD patterns:
- Use route overrides in the gateway configuration
- Example: `VerifyEmail` → POST `/api/users/:id/verify-email` (manual override)

## Examples from Real Services

### User Service
```protobuf
service UserService {
  // Simple CRUD
  rpc GetUser(GetUserRequest) returns (UserResponse);
  // → GET /api/users/:id

  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  // → GET /api/users

  rpc CreateUser(CreateUserRequest) returns (UserResponse);
  // → POST /api/users

  rpc UpdateUser(UpdateUserRequest) returns (UserResponse);
  // → PUT /api/users/:id

  rpc DeleteUser(DeleteUserRequest) returns (google.protobuf.Empty);
  // → DELETE /api/users/:id

  // Nested resources
  rpc AddGroupMember(AddGroupMemberRequest) returns (google.protobuf.Empty);
  // → POST /api/groups/:id/members

  rpc RemoveGroupMember(RemoveGroupMemberRequest) returns (google.protobuf.Empty);
  // → DELETE /api/groups/:id/members/:user_id

  rpc GetGroupMembers(GetGroupMembersRequest) returns (GetGroupMembersResponse);
  // → GET /api/groups/:id/members

  // Custom actions (require manual override)
  rpc Login(LoginRequest) returns (LoginResponse);
  rpc Register(RegisterRequest) returns (RegisterResponse);
  rpc ChangePassword(ChangePasswordRequest) returns (google.protobuf.Empty);
}
```

## Best Practices

1. **Follow the convention strictly** - This enables automatic route discovery
2. **Use descriptive resource names** - Clear names improve API understanding
3. **Keep nesting shallow** - Prefer 1-2 levels of nesting maximum
4. **Use singular form in method names** - Except for `List{Resources}`
5. **Use manual overrides sparingly** - Only for truly exceptional cases
6. **Document deviations** - If you must deviate, document why

## Configuration

### Enabling Auto-Discovery

In your gateway configuration:

```yaml
services:
  user-service:
    endpoint: "user-service:50051"
    auto_discover: true  # Enable automatic route discovery
```

### Manual Overrides

For methods that don't fit the convention:

```yaml
route_overrides:
  - grpc_method: "user.UserService/Login"
    http_method: "POST"
    http_path: "/api/auth/login"

  - grpc_method: "user.UserService/Register"
    http_method: "POST"
    http_path: "/api/auth/register"
```

## Testing Your Routes

After defining your gRPC methods, you can verify the generated routes by:

1. Starting the API Gateway with discovery enabled
2. Checking the logs for route mapping messages
3. Testing the routes with curl or your API client

Look for log messages like:
```
✅ CONVENTION: Generated HTTP route mapping
   method_name = "RemoveGroupMember"
   http_method = "DELETE"
   http_path = "/api/groups/:id/members/:user_id"
   grpc_method = "user.UserService/RemoveGroupMember"
```

## Summary

By following these naming conventions, you can:
- ✅ Eliminate manual route configuration
- ✅ Maintain RESTful API design
- ✅ Ensure consistency across services
- ✅ Enable automatic API documentation
- ✅ Simplify service development

For questions or suggestions, please refer to the API Gateway documentation or contact the platform team.
