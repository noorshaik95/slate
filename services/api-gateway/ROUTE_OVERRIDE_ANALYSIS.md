# Route Override Analysis

This document explains the analysis performed on gateway route overrides and why certain patterns were implemented while others were not.

## Summary Statistics

### Before Optimization
- **Total route overrides**: 82

### After Optimization (Current)
- **Total route overrides**: 58
- **Routes auto-discovered**: 24
- **Reduction**: 29% fewer manual overrides

## Implemented Patterns

### 1. Simple CRUD Operations (Tier 1)
**Pattern**: `{Get|List|Create|Update|Delete}{Resource}`

**Routes removed** (12 total):
- Course: CreateCourse, GetCourse, UpdateCourse, DeleteCourse, ListCourses
- Assignment: CreateAssignment, GetAssignment, UpdateAssignment, DeleteAssignment, ListAssignments
- Other: GetUser, etc.

**Impact**: These are the foundation of REST APIs and were the highest priority.

### 2. Nested Resource Operations (Tier 3)
**Pattern**: `{Add|Remove}{Parent}{Child}` and `{Get|List}{Parent}{Children}`

**Routes removed** (7 total):
- User Groups: AddGroupMember, RemoveGroupMember, GetGroupMembers, GetUserGroups
- Course Enrollments: GetStudentEnrollments
- Course Sections: ListCourseSections
- Other nested patterns

**Impact**: Handles parent-child resource relationships common in REST APIs.

### 3. Custom Action Operations (Tier 2)
**Pattern**: `{Publish|Unpublish}{Resource}`

**Routes removed** (5 total):
- PublishCourse, UnpublishCourse
- PublishGrade
- PublishContent, UnpublishContent

**Impact**: Standardizes publish/unpublish actions across all resources.

## Patterns NOT Implemented

### Analysis Methodology

We analyzed the remaining 58 route overrides to identify potential patterns. A pattern is worth implementing if:
1. It appears **3+ times** across different services
2. It has a **consistent path structure**
3. It represents a **common REST pattern** (not business-specific logic)

### Frequency Analysis

| Action Pattern | Occurrences | Worth Implementing? | Reason |
|---------------|-------------|---------------------|--------|
| Publish/Unpublish | 5 | ✅ **Implemented** | High frequency, consistent structure |
| Submit | 1 | ❌ No | Single occurrence |
| Export | 1 | ❌ No | Single occurrence |
| Check | 1 | ❌ No | Single occurrence |
| Initiate | 1 | ❌ No | Single occurrence |
| Upload | 1 | ❌ No | Single occurrence, complex multi-step flow |
| Complete | 1 | ❌ No | Single occurrence |
| Cancel | 1 | ❌ No | Single occurrence |
| Mark | 1 | ❌ No | Single occurrence |
| Reorder | 1 | ❌ No | Single occurrence |
| Search | 1 | ❌ No | Single occurrence |
| Generate | 1 | ❌ No | Single occurrence |
| Enroll | 1 | ❌ No | Single occurrence, custom business logic |

### Specific Examples

#### Authentication Routes (12 routes)
**Methods**: Login, Register, RefreshToken, ValidateToken, Logout, etc.

**Why not a pattern?**
- These are security-sensitive endpoints with custom paths (`/api/auth/*`)
- Path structure intentionally differs from resource-based REST
- Business logic requires explicit configuration

**Verdict**: Keep as manual overrides ✅

#### OAuth/SAML Routes (8 routes)
**Methods**: OAuthCallback, LinkOAuthProvider, GetOAuthAuthorizationURL, etc.

**Why not a pattern?**
- Complex authentication flows with specific callback URLs
- Integration with external providers requires precise path control
- Security and compliance require explicit configuration

**Verdict**: Keep as manual overrides ✅

#### MFA Routes (5 routes)
**Methods**: SetupMFA, VerifyMFA, DisableMFA, GetMFAStatus, ValidateMFACode

**Why not a pattern?**
- Only appears in one service (user-auth)
- Security-sensitive operations requiring explicit paths
- Custom path structure (`/api/mfa/*`) for security grouping

**Verdict**: Keep as manual overrides ✅

#### Upload Routes (4 routes)
**Methods**: InitiateUpload, UploadChunk, CompleteUpload, CancelUpload

**Why not a pattern?**
- Represents a specific multi-step file upload protocol
- Not a general REST pattern, but a specialized workflow
- Paths intentionally grouped under `/api/content/upload/*`

**Verdict**: Keep as manual overrides ✅

#### Special Get Methods with Custom Hierarchies
**Methods**: GetStudentGradebook, GetCourseGradebook, GetGradeStatistics, etc.

**Why not a pattern?**
- Each has a unique path structure based on business requirements
- Resource hierarchy doesn't follow compound naming (e.g., "StudentGradebook" would split incorrectly)
- Using custom parameter names (`:student_id` instead of `:id`) for clarity

**Example**:
```
GetStudentGradebook
  Expected:  /api/students/:student_id/gradebook
  Auto-gen:  /api/students/:id/gradebooks (doesn't match)
```

**Verdict**: Keep as manual overrides ✅

#### Nested Resources That Don't Split Well

Some methods appear to be nested resources but don't follow the compound naming convention:

**Examples**:
- `AddPrerequisite` → Cannot split "Prerequisite" into parent/child
- `AddCoInstructor` → Splits as "Co" + "Instructor" (incorrect)
- `GetCrossListedCourses` → Splits as "Cross" + "ListedCourses" (incorrect)

**Why not fix the splitting logic?**
- These represent **business domain concepts**, not resource hierarchies
- Making the splitter more complex would create false positives
- Better to be explicit about business-specific routes

**Verdict**: Keep as manual overrides ✅

## Routes That Could Work But Don't (Edge Cases)

Some routes are close to working with auto-discovery but fail due to subtle differences:

### GetCourseRoster
- **Expected**: `/api/courses/:id/roster` (singular)
- **Generated**: `/api/courses/:id/rosters` (plural)
- **Issue**: Roster is used in singular form in the API

### Gradebook Routes
- **Expected**: `/api/students/:student_id/gradebook` (singular)
- **Generated**: `/api/students/:id/gradebooks` (plural)
- **Issue**: Uses singular "gradebook" and custom param `:student_id`

**Decision**: Don't add special cases for singular/plural edge cases. Manual overrides are clearer.

## Recommendations

### For Future Development

1. **Follow naming conventions** when adding new gRPC methods:
   - Use `Publish{Resource}` for publish actions
   - Use `{Add|Remove}{Parent}{Child}` for nested resource operations
   - Use standard CRUD: `{Get|List|Create|Update|Delete}{Resource}`

2. **When to use manual overrides**:
   - Authentication/authorization endpoints
   - Complex multi-step workflows (uploads, OAuth flows)
   - Business-specific actions that appear only once
   - When you need custom parameter names for clarity

3. **Patterns to avoid implementing**:
   - Single-occurrence actions (not worth the complexity)
   - Business-specific operations (better to be explicit)
   - Routes requiring special casing or conditional logic

## Conclusion

The current implementation strikes a good balance:
- **24 routes auto-discovered** (29% reduction)
- **3 pattern tiers** cover common REST conventions
- **58 manual overrides** remaining are legitimately custom

Further pattern additions would yield diminishing returns and risk over-complicating the convention system. The remaining overrides are intentionally explicit for security, clarity, and business logic specificity.

## Changelog

- **2025-11-18**: Initial pattern implementation
  - Added Simple CRUD pattern (Tier 1)
  - Added Nested Resource pattern (Tier 3)
  - Added Publish/Unpublish pattern (Tier 2)
  - Removed 24 route overrides
  - Created this analysis document
