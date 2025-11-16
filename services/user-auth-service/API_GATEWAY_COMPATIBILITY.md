# API Gateway Auto-Discovery Compatibility

This document shows which routes auto-discover via naming conventions and which require manual overrides.

## API Gateway Naming Convention Rules

The gateway supports **5 naming patterns** for auto-discovery:
- `Get` → GET /api/{resources}/:id
- `List` → GET /api/{resources}
- `Create` → POST /api/{resources}
- `Update` → PUT /api/{resources}/:id
- `Delete` → DELETE /api/{resources}/:id

Any method NOT matching these patterns requires a manual route override.

---

## Authentication & User Management

### ✅ Auto-Discovered (RESTful patterns)

| gRPC Method | HTTP Route | Method | Pattern |
|-------------|------------|--------|---------|
| `GetUser` | `/api/users/:id` | GET | Get |
| `ListUsers` | `/api/users` | GET | List |
| `CreateUser` | `/api/users` | POST | Create |
| `UpdateUser` | `/api/users/:id` | PUT | Update |
| `DeleteUser` | `/api/users/:id` | DELETE | Delete |
| `GetProfile` | `/api/profiles/:id` | GET | Get |
| `UpdateProfile` | `/api/profiles/:id` | PUT | Update |
| `GetUserRoles` | `/api/userroles/:id` | GET | Get |
| `AssignRole` | N/A | N/A | ❌ Needs override |
| `RemoveRole` | N/A | N/A | ❌ Needs override |
| `CheckPermission` | N/A | N/A | ❌ Needs override |

### ❌ Manual Overrides (Non-RESTful)

| gRPC Method | HTTP Route | Method | Reason |
|-------------|------------|--------|--------|
| `Login` | `/api/auth/login` | POST | Not RESTful |
| `Register` | `/api/auth/register` | POST | Not RESTful |
| `RefreshToken` | `/api/auth/refresh` | POST | Not RESTful |
| `ValidateToken` | `/api/auth/validate` | POST | Not RESTful |
| `Logout` | `/api/auth/logout` | POST | Not RESTful |
| `ChangePassword` | N/A | N/A | TODO: Add override |

**Public Routes:**
- ✅ `/api/auth/login` (POST)
- ✅ `/api/auth/register` (POST)
- ✅ `/api/auth/refresh` (POST)

---

## OAuth/SSO Management

### ❌ All Require Manual Overrides

| gRPC Method | HTTP Route | Method | Configured |
|-------------|------------|--------|------------|
| `OAuthCallback` | `/api/oauth/callback` | POST | ✅ Yes |
| `LinkOAuthProvider` | `/api/oauth/link` | POST | ✅ Yes |
| `UnlinkOAuthProvider` | `/api/oauth/unlink` | POST | ✅ Yes |
| `GetOAuthProviders` | `/api/oauth/providers` | GET | ✅ Yes |

**Public Routes:**
- ✅ `/api/oauth/callback` (POST) - Required for OAuth flow

**Reasoning:** OAuth methods don't follow CRUD patterns

---

## MFA/2FA Management

### ❌ All Require Manual Overrides

| gRPC Method | HTTP Route | Method | Configured |
|-------------|------------|--------|------------|
| `SetupMFA` | `/api/mfa/setup` | POST | ✅ Yes |
| `VerifyMFA` | `/api/mfa/verify` | POST | ✅ Yes |
| `DisableMFA` | `/api/mfa/disable` | POST | ✅ Yes |
| `GetMFAStatus` | `/api/mfa/status` | GET | ✅ Yes |
| `ValidateMFACode` | `/api/mfa/validate` | POST | ✅ Yes |

**Public Routes:** None (all require authentication)

**Reasoning:** MFA methods are action-oriented, not resource CRUD

---

## User Groups Management

### ✅ Auto-Discovered (CRUD operations)

| gRPC Method | HTTP Route | Method | Pattern |
|-------------|------------|--------|---------|
| `CreateGroup` | `/api/groups` | POST | Create ✅ |
| `GetGroup` | `/api/groups/:id` | GET | Get ✅ |
| `UpdateGroup` | `/api/groups/:id` | PUT | Update ✅ |
| `DeleteGroup` | `/api/groups/:id` | DELETE | Delete ✅ |
| `ListGroups` | `/api/groups` | GET | List ✅ |

### ❌ Manual Overrides (Member management)

| gRPC Method | HTTP Route | Method | Configured |
|-------------|------------|--------|------------|
| `AddGroupMember` | `/api/groups/:id/members` | POST | ✅ Yes |
| `RemoveGroupMember` | `/api/groups/:id/members/:user_id` | DELETE | ✅ Yes |
| `GetGroupMembers` | `/api/groups/:id/members` | GET | ✅ Yes |
| `GetUserGroups` | `/api/users/:id/groups` | GET | ✅ Yes |

**Public Routes:** None (all require authentication)

**Reasoning:** Member management is nested resource operations

---

## Parent-Child Account Management

### ❌ All Require Manual Overrides

| gRPC Method | HTTP Route | Method | Configured |
|-------------|------------|--------|------------|
| `CreateParentChildLink` | `/api/parent-child/link` | POST | ✅ Yes |
| `RemoveParentChildLink` | `/api/parent-child/link` | DELETE | ✅ Yes |
| `GetChildAccounts` | `/api/users/:id/children` | GET | ✅ Yes |
| `GetParentAccounts` | `/api/users/:id/parents` | GET | ✅ Yes |
| `UpdateParentChildPermissions` | `/api/parent-child/permissions` | PUT | ✅ Yes |

**Public Routes:** None (all require authentication and authorization)

**Reasoning:** Relationship management doesn't follow standard CRUD patterns

---

## Summary Statistics

### Auto-Discovery Success Rate

| Category | Total Methods | Auto-Discovered | Manual Overrides | Success Rate |
|----------|---------------|-----------------|------------------|--------------|
| **Users & Auth** | 11 | 7 | 4 | 64% |
| **OAuth** | 4 | 0 | 4 | 0% |
| **MFA** | 5 | 0 | 5 | 0% |
| **Groups** | 9 | 5 | 4 | 56% |
| **Parent-Child** | 5 | 0 | 5 | 0% |
| **TOTAL** | 34 | 12 | 22 | 35% |

### Route Override Configuration Status

**Total Overrides Configured:** 22 ✅
- Authentication: 5 routes
- OAuth: 4 routes
- MFA: 5 routes
- Groups (members): 4 routes
- Parent-Child: 5 routes

### Public Routes Configured

**Total Public Routes:** 4
- `/api/auth/login` (POST)
- `/api/auth/register` (POST)
- `/api/auth/refresh` (POST)
- `/api/oauth/callback` (POST)

---

## Testing Routes

### Test Auto-Discovered Routes

```bash
# Users (auto-discovered)
curl http://localhost:8080/api/users
curl http://localhost:8080/api/users/user-123

# Groups (auto-discovered)
curl http://localhost:8080/api/groups
curl http://localhost:8080/api/groups/group-123
```

### Test Override Routes

```bash
# Authentication
curl -X POST http://localhost:8080/api/auth/login -d '{"email":"user@example.com","password":"pass"}'

# OAuth
curl -X POST http://localhost:8080/api/oauth/callback -d '{"provider":"google","code":"xyz"}'

# MFA
curl -X POST http://localhost:8080/api/mfa/setup -H "Authorization: Bearer $TOKEN"

# Groups (members)
curl -X POST http://localhost:8080/api/groups/group-123/members -d '{"user_id":"user-123","role":"member"}'

# Parent-Child
curl http://localhost:8080/api/users/user-123/children -H "Authorization: Bearer $TOKEN"
```

---

## Gateway Discovery Logs

When the gateway starts, it will log route discovery:

```
INFO Starting route discovery from all services
INFO service="user-auth-service" Successfully discovered routes from service routes=12
INFO total_routes=12 services_success=1 services_failed=0 Route discovery completed
INFO overrides_configured=22 Applied route overrides
INFO total_routes=34 Routes ready (including overrides)
```

The final route count should be **34 routes** (12 auto-discovered + 22 overrides).

---

## Notes

1. **Auto-Discovery Advantages:**
   - Zero configuration for CRUD operations
   - Automatic updates when methods are added
   - Follows REST conventions

2. **When to Use Overrides:**
   - Non-CRUD operations (actions, commands)
   - Custom path structures
   - Public endpoints (authentication bypass)
   - Nested resources

3. **Design Recommendation:**
   - ✅ Use auto-discovery for resource CRUD
   - ✅ Use overrides for actions and workflows
   - ✅ Keep override paths RESTful when possible
   - ✅ Document all overrides in this file
