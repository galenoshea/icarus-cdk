# Data Manager Template

A production-ready template for managing structured data with full CRUD operations, search capabilities, and user management.

## Features

### ✨ Core Functionality
- **Full CRUD Operations**: Create, read, update, delete records
- **Advanced Search**: Text search with category, tag, and date filtering
- **User Management**: Role-based access control (Admin, Editor, Viewer)
- **Data Analytics**: Real-time statistics and reporting
- **Data Export**: JSON export for backup and portability

### 🔒 Security & Access Control
- **Principal-based Authentication**: Uses ICP identity system
- **Role-based Permissions**: Admin, Editor, Viewer roles
- **Privacy Controls**: Public/private record visibility
- **Input Validation**: Comprehensive data validation

### 📊 Advanced Features
- **Indexed Search**: Fast category and tag-based queries
- **Activity Logging**: Track all user actions
- **Analytics Dashboard**: Usage statistics and insights
- **Configurable Limits**: Per-user record limits and quotas

## Quick Start

1. **Create from Template**:
   ```bash
   icarus new my-data-app --template data-manager
   cd my-data-app
   ```

2. **Deploy Locally**:
   ```bash
   dfx start --background
   icarus build
   icarus deploy --network local
   ```

3. **Connect to Claude Desktop**:
   ```json
   {
     "mcpServers": {
       "my-data-app": {
         "command": "icarus",
         "args": ["bridge", "start", "--canister-id", "YOUR_CANISTER_ID"]
       }
     }
   }
   ```

## Available MCP Tools

### Record Management
- `create_record` - Create a new data record with metadata
- `get_record` - Retrieve a record by ID (with permission checks)
- `update_record` - Update existing record (owner/admin only)
- `delete_record` - Delete a record (owner/admin only)

### Search & Query
- `search_records` - Advanced search with filtering and pagination
- `get_my_records` - Get all records owned by current user

### User Management
- `get_profile` - Get current user's profile and preferences
- `update_preferences` - Update user preferences and settings

### Analytics & Reporting
- `get_analytics` - Get system analytics (admin only)
- `export_data` - Export user's data as JSON

## Usage Examples

### Creating Records
```javascript
// In Claude Desktop
Human: Create a new project record with title "Website Redesign", content "Redesign company website with modern UI", category "Projects", and tags ["web", "design", "priority-high"]

Claude: I'll create that project record for you.
[Uses create_record tool]
✅ Record created successfully with ID: abc-123-def
```

### Searching Data
```javascript
Human: Search for all records in the "Projects" category that contain "design" and were created in the last 30 days

Claude: I'll search for project records matching your criteria.
[Uses search_records tool with filters]
Found 5 matching records:
1. Website Redesign (ID: abc-123-def)
2. Mobile App Design (ID: def-456-ghi)
...
```

### Analytics
```javascript
Human: Show me the analytics dashboard

Claude: Here's your data analytics:
[Uses get_analytics tool - admin only]
📊 Total Records: 1,247
📂 Top Categories: Projects (45%), Documents (30%), Resources (25%)
🏷️ Popular Tags: design (89), development (67), research (45)
```

## Data Models

### DataRecord
```rust
pub struct DataRecord {
    pub id: String,           // UUID
    pub title: String,        // Record title
    pub content: String,      // Main content
    pub category: String,     // Categorization
    pub tags: Vec<String>,    // Searchable tags
    pub metadata: HashMap<String, String>, // Custom fields
    pub created_at: u64,      // Timestamp
    pub updated_at: u64,      // Last modified
    pub created_by: Principal, // Owner
    pub is_public: bool,      // Visibility
}
```

### UserProfile
```rust
pub struct UserProfile {
    pub principal: Principal,
    pub username: String,
    pub role: UserRole,       // Admin/Editor/Viewer
    pub preferences: UserPreferences,
    pub created_at: u64,
    pub last_active: u64,
}
```

## Configuration

Customize the application in `src/lib.rs`:

```rust
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_records_per_user: 1000,     // Increase/decrease limits
            max_file_size_mb: 10,           // Content size limits
            allowed_categories: vec![       // Define your categories
                "General".to_string(),
                "Documents".to_string(),
                "Projects".to_string(),
                "Resources".to_string(),
            ],
            require_approval: false,        // Enable content moderation
            backup_enabled: true,           // Auto-backup features
        }
    }
}
```

## Permission System

### Role Hierarchy
- **Admin**: Full access, can manage all records and users
- **Editor**: Can create, read, update own records and public records
- **Viewer**: Can only read public records

### Permission Matrix
| Action | Admin | Editor | Viewer |
|--------|-------|--------|---------|
| Create Record | ✅ | ✅ | ❌ |
| Read Own Records | ✅ | ✅ | ✅ |
| Read Public Records | ✅ | ✅ | ✅ |
| Read All Records | ✅ | ❌ | ❌ |
| Update Own Records | ✅ | ✅ | ❌ |
| Update Any Record | ✅ | ❌ | ❌ |
| Delete Own Records | ✅ | ✅ | ❌ |
| Delete Any Record | ✅ | ❌ | ❌ |
| View Analytics | ✅ | ❌ | ❌ |

## Storage Architecture

### Memory Layout
- **Memory 0**: Core data (records, users, activity log)
- **Memory 1**: Search indexes (category, tag, user indexes)
- **Memory 2**: Configuration and cached analytics

### Indexing Strategy
- **Category Index**: Fast filtering by category
- **Tag Index**: Efficient tag-based search
- **User Index**: Quick access to user's records
- **Activity Log**: Audit trail for all operations

## Advanced Customization

### Adding Custom Fields
```rust
#[derive(CandidType, Serialize, Deserialize)]
pub struct CustomRecordArgs {
    // Standard fields
    pub title: String,
    pub content: String,

    // Add your custom fields
    pub priority: Priority,
    pub due_date: Option<u64>,
    pub assignee: Option<Principal>,
    pub status: Status,
}
```

### Custom Search Filters
```rust
#[icarus_tool("Search by status and priority")]
pub async fn search_by_status(
    status: Status,
    priority: Option<Priority>
) -> Result<Vec<DataRecord>, String> {
    // Custom search implementation
}
```

### Webhook Integration
```rust
#[icarus_tool("Configure webhook notifications")]
pub async fn setup_webhook(webhook_url: String) -> Result<String, String> {
    // Send notifications to external services
}
```

## Production Deployment

1. **Configure for Scale**:
   ```rust
   // Adjust limits for production
   max_records_per_user: 10000,
   max_file_size_mb: 100,
   backup_enabled: true,
   ```

2. **Set Up Monitoring**:
   ```bash
   # Monitor canister health
   icarus status --canister-id YOUR_ID --network ic
   ```

3. **Backup Strategy**:
   - Enable automatic exports
   - Set up regular data exports
   - Monitor storage usage

## License

This template is provided under the BSL-1.1 license. See the main SDK license for details.