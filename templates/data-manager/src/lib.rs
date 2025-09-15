//! Production-Ready Data Manager Template
//!
//! This template demonstrates best practices for managing structured data on ICP:
//! - Full CRUD operations with validation
//! - Search and filtering capabilities
//! - Analytics and reporting
//! - User-based access control
//! - Data export and backup

use icarus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// === Data Models ===

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub struct DataRecord {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: Principal,
    pub is_public: bool,
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub struct UserProfile {
    pub principal: Principal,
    pub username: String,
    pub role: UserRole,
    pub preferences: UserPreferences,
    pub created_at: u64,
    pub last_active: u64,
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug, PartialEq)]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub struct UserPreferences {
    pub items_per_page: u32,
    pub default_category: String,
    pub email_notifications: bool,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct CreateRecordArgs {
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub is_public: bool,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct UpdateRecordArgs {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub is_public: Option<bool>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub records: Vec<DataRecord>,
    pub total_count: u64,
    pub has_more: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Analytics {
    pub total_records: u64,
    pub records_by_category: HashMap<String, u64>,
    pub records_by_user: HashMap<String, u64>,
    pub recent_activity: Vec<ActivityEntry>,
    pub popular_tags: Vec<(String, u64)>,
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub struct ActivityEntry {
    pub timestamp: u64,
    pub user: Principal,
    pub action: String,
    pub resource_id: String,
}

// === Stable Storage ===

stable_storage! {
    memory 0: {
        // Primary data storage
        records: Map<String, DataRecord> = Map::init();

        // User management
        users: Map<Principal, UserProfile> = Map::init();

        // Analytics and activity tracking
        activity_log: Vec<ActivityEntry> = Vec::init();
    },

    memory 1: {
        // Indexes for efficient querying
        category_index: Map<String, Vec<String>> = Map::init();
        tag_index: Map<String, Vec<String>> = Map::init();
        user_records_index: Map<Principal, Vec<String>> = Map::init();
    },

    memory 2: {
        // Configuration and metadata
        app_config: Cell<AppConfig> = Cell::init(AppConfig::default());
        stats_cache: Cell<Analytics> = Cell::init(Analytics::default());
    }
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub struct AppConfig {
    pub max_records_per_user: u32,
    pub max_file_size_mb: u32,
    pub allowed_categories: Vec<String>,
    pub require_approval: bool,
    pub backup_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_records_per_user: 1000,
            max_file_size_mb: 10,
            allowed_categories: vec![
                "General".to_string(),
                "Documents".to_string(),
                "Projects".to_string(),
                "Resources".to_string(),
            ],
            require_approval: false,
            backup_enabled: true,
        }
    }
}

impl Default for Analytics {
    fn default() -> Self {
        Self {
            total_records: 0,
            records_by_category: HashMap::new(),
            records_by_user: HashMap::new(),
            recent_activity: Vec::new(),
            popular_tags: Vec::new(),
        }
    }
}

// === MCP Tools ===

#[icarus_module]
mod data_manager {
    use super::*;

    // === Record Management ===

    #[icarus_tool("Create a new data record")]
    pub async fn create_record(args: CreateRecordArgs) -> Result<String, String> {
        let caller = ic_cdk::caller();

        // Validate user permissions
        let user = get_or_create_user(caller).await?;
        if user.role == UserRole::Viewer {
            return Err("Insufficient permissions to create records".to_string());
        }

        // Validate input
        if args.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if args.content.len() > 50_000 {
            return Err("Content exceeds maximum length".to_string());
        }

        // Check user limits
        let user_record_count = get_user_record_count(caller).await;
        let config = STORAGE.with(|s| s.borrow().app_config.get().clone());
        if user_record_count >= config.max_records_per_user as usize {
            return Err(format!("Maximum records limit reached ({})", config.max_records_per_user));
        }

        // Validate category
        if !config.allowed_categories.contains(&args.category) {
            return Err(format!("Invalid category. Allowed: {:?}", config.allowed_categories));
        }

        let record_id = Uuid::new_v4().to_string();
        let now = ic_cdk::api::time();

        let record = DataRecord {
            id: record_id.clone(),
            title: args.title.trim().to_string(),
            content: args.content,
            category: args.category.clone(),
            tags: args.tags.clone(),
            metadata: args.metadata,
            created_at: now,
            updated_at: now,
            created_by: caller,
            is_public: args.is_public,
        };

        STORAGE.with(|s| {
            let mut storage = s.borrow_mut();

            // Store record
            storage.records.insert(record_id.clone(), record);

            // Update indexes
            update_category_index(&mut storage, &args.category, &record_id);
            update_tag_index(&mut storage, &args.tags, &record_id);
            update_user_index(&mut storage, caller, &record_id);

            // Log activity
            let activity = ActivityEntry {
                timestamp: now,
                user: caller,
                action: "create_record".to_string(),
                resource_id: record_id.clone(),
            };
            storage.activity_log.push(activity);
        });

        // Update analytics cache
        update_analytics_cache().await;

        Ok(record_id)
    }

    #[icarus_tool("Get a record by ID")]
    pub async fn get_record(id: String) -> Result<DataRecord, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let storage = s.borrow();

            match storage.records.get(&id) {
                Some(record) => {
                    // Check permissions
                    if !record.is_public && record.created_by != caller {
                        // Check if user is admin
                        if let Some(user) = storage.users.get(&caller) {
                            if user.role != UserRole::Admin {
                                return Err("Access denied".to_string());
                            }
                        } else {
                            return Err("Access denied".to_string());
                        }
                    }
                    Ok(record)
                },
                None => Err("Record not found".to_string()),
            }
        })
    }

    #[icarus_tool("Update an existing record")]
    pub async fn update_record(args: UpdateRecordArgs) -> Result<DataRecord, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let mut storage = s.borrow_mut();

            let mut record = match storage.records.get(&args.id) {
                Some(record) => record,
                None => return Err("Record not found".to_string()),
            };

            // Check permissions
            if record.created_by != caller {
                if let Some(user) = storage.users.get(&caller) {
                    if user.role != UserRole::Admin && user.role != UserRole::Editor {
                        return Err("Insufficient permissions to update this record".to_string());
                    }
                } else {
                    return Err("Access denied".to_string());
                }
            }

            let now = ic_cdk::api::time();
            let mut updated_tags = false;
            let mut updated_category = false;

            // Apply updates
            if let Some(title) = args.title {
                if title.trim().is_empty() {
                    return Err("Title cannot be empty".to_string());
                }
                record.title = title.trim().to_string();
            }

            if let Some(content) = args.content {
                if content.len() > 50_000 {
                    return Err("Content exceeds maximum length".to_string());
                }
                record.content = content;
            }

            if let Some(category) = args.category {
                let config = storage.app_config.get();
                if !config.allowed_categories.contains(&category) {
                    return Err(format!("Invalid category. Allowed: {:?}", config.allowed_categories));
                }
                record.category = category;
                updated_category = true;
            }

            if let Some(tags) = args.tags {
                record.tags = tags;
                updated_tags = true;
            }

            if let Some(metadata) = args.metadata {
                record.metadata = metadata;
            }

            if let Some(is_public) = args.is_public {
                record.is_public = is_public;
            }

            record.updated_at = now;

            // Update indexes if needed
            if updated_category {
                update_category_index(&mut storage, &record.category, &record.id);
            }
            if updated_tags {
                update_tag_index(&mut storage, &record.tags, &record.id);
            }

            // Store updated record
            storage.records.insert(args.id.clone(), record.clone());

            // Log activity
            let activity = ActivityEntry {
                timestamp: now,
                user: caller,
                action: "update_record".to_string(),
                resource_id: args.id,
            };
            storage.activity_log.push(activity);

            Ok(record)
        })
    }

    #[icarus_tool("Delete a record")]
    pub async fn delete_record(id: String) -> Result<String, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let mut storage = s.borrow_mut();

            let record = match storage.records.get(&id) {
                Some(record) => record,
                None => return Err("Record not found".to_string()),
            };

            // Check permissions
            if record.created_by != caller {
                if let Some(user) = storage.users.get(&caller) {
                    if user.role != UserRole::Admin {
                        return Err("Insufficient permissions to delete this record".to_string());
                    }
                } else {
                    return Err("Access denied".to_string());
                }
            }

            // Remove from storage
            storage.records.remove(&id);

            // Clean up indexes
            remove_from_category_index(&mut storage, &record.category, &id);
            remove_from_tag_index(&mut storage, &record.tags, &id);
            remove_from_user_index(&mut storage, caller, &id);

            // Log activity
            let activity = ActivityEntry {
                timestamp: ic_cdk::api::time(),
                user: caller,
                action: "delete_record".to_string(),
                resource_id: id.clone(),
            };
            storage.activity_log.push(activity);

            Ok(format!("Record {} deleted successfully", id))
        })
    }

    // === Search and Query ===

    #[icarus_tool("Search records with advanced filtering")]
    pub async fn search_records(query: SearchQuery) -> Result<SearchResult, String> {
        let caller = ic_cdk::caller();
        let limit = query.limit.unwrap_or(50).min(100) as usize;
        let offset = query.offset.unwrap_or(0) as usize;

        STORAGE.with(|s| {
            let storage = s.borrow();
            let mut matched_records = Vec::new();

            // Get user role for permission checking
            let user_role = storage.users.get(&caller).map(|u| u.role.clone());

            for record in storage.records.iter() {
                // Permission check
                if !record.1.is_public && record.1.created_by != caller {
                    if user_role != Some(UserRole::Admin) {
                        continue;
                    }
                }

                // Apply filters
                if let Some(category) = &query.category {
                    if &record.1.category != category {
                        continue;
                    }
                }

                if !query.tags.is_empty() {
                    if !query.tags.iter().any(|tag| record.1.tags.contains(tag)) {
                        continue;
                    }
                }

                if let Some(after) = query.created_after {
                    if record.1.created_at < after {
                        continue;
                    }
                }

                if let Some(before) = query.created_before {
                    if record.1.created_at > before {
                        continue;
                    }
                }

                // Text search
                if !query.query.is_empty() {
                    let search_text = format!("{} {} {}", record.1.title, record.1.content, record.1.category).to_lowercase();
                    let query_lower = query.query.to_lowercase();
                    if !search_text.contains(&query_lower) {
                        continue;
                    }
                }

                matched_records.push(record.1.clone());
            }

            // Sort by update time (newest first)
            matched_records.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

            let total_count = matched_records.len() as u64;
            let has_more = total_count > (offset + limit) as u64;

            // Apply pagination
            let records: Vec<DataRecord> = matched_records
                .into_iter()
                .skip(offset)
                .take(limit)
                .collect();

            Ok(SearchResult {
                records,
                total_count,
                has_more,
            })
        })
    }

    #[icarus_tool("Get user's own records")]
    pub async fn get_my_records() -> Result<Vec<DataRecord>, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let storage = s.borrow();

            if let Some(record_ids) = storage.user_records_index.get(&caller) {
                let records: Vec<DataRecord> = record_ids
                    .iter()
                    .filter_map(|id| storage.records.get(id))
                    .collect();
                Ok(records)
            } else {
                Ok(Vec::new())
            }
        })
    }

    // === Analytics and Reporting ===

    #[icarus_tool("Get analytics and statistics")]
    pub async fn get_analytics() -> Result<Analytics, String> {
        let caller = ic_cdk::caller();

        // Check permissions
        let has_access = STORAGE.with(|s| {
            let storage = s.borrow();
            storage.users.get(&caller)
                .map(|user| user.role == UserRole::Admin)
                .unwrap_or(false)
        });

        if !has_access {
            return Err("Admin access required".to_string());
        }

        update_analytics_cache().await;

        STORAGE.with(|s| {
            Ok(s.borrow().stats_cache.get().clone())
        })
    }

    // === User Management ===

    #[icarus_tool("Get user profile")]
    pub async fn get_profile() -> Result<UserProfile, String> {
        let caller = ic_cdk::caller();
        get_or_create_user(caller).await
    }

    #[icarus_tool("Update user preferences")]
    pub async fn update_preferences(prefs: UserPreferences) -> Result<UserProfile, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let mut storage = s.borrow_mut();

            let mut user = match storage.users.get(&caller) {
                Some(user) => user,
                None => return Err("User not found".to_string()),
            };

            user.preferences = prefs;
            user.last_active = ic_cdk::api::time();

            storage.users.insert(caller, user.clone());
            Ok(user)
        })
    }

    // === Data Export and Backup ===

    #[icarus_tool("Export user's data as JSON")]
    pub async fn export_data() -> Result<String, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let storage = s.borrow();

            let user_records: Vec<DataRecord> = storage.user_records_index
                .get(&caller)
                .unwrap_or_default()
                .iter()
                .filter_map(|id| storage.records.get(id))
                .collect();

            let export_data = serde_json::json!({
                "user": storage.users.get(&caller),
                "records": user_records,
                "export_timestamp": ic_cdk::api::time(),
                "total_records": user_records.len()
            });

            serde_json::to_string_pretty(&export_data)
                .map_err(|e| format!("Failed to serialize data: {}", e))
        })
    }
}

// === Helper Functions ===

async fn get_or_create_user(principal: Principal) -> Result<UserProfile, String> {
    STORAGE.with(|s| {
        let mut storage = s.borrow_mut();

        if let Some(user) = storage.users.get(&principal) {
            Ok(user)
        } else {
            // Create new user
            let now = ic_cdk::api::time();
            let user = UserProfile {
                principal,
                username: principal.to_string()[..8].to_string(), // First 8 chars as username
                role: UserRole::Editor, // Default role
                preferences: UserPreferences {
                    items_per_page: 20,
                    default_category: "General".to_string(),
                    email_notifications: true,
                },
                created_at: now,
                last_active: now,
            };

            storage.users.insert(principal, user.clone());
            Ok(user)
        }
    })
}

async fn get_user_record_count(principal: Principal) -> usize {
    STORAGE.with(|s| {
        let storage = s.borrow();
        storage.user_records_index
            .get(&principal)
            .map(|records| records.len())
            .unwrap_or(0)
    })
}

async fn update_analytics_cache() {
    STORAGE.with(|s| {
        let mut storage = s.borrow_mut();

        let mut analytics = Analytics {
            total_records: storage.records.len() as u64,
            records_by_category: HashMap::new(),
            records_by_user: HashMap::new(),
            recent_activity: storage.activity_log.iter().rev().take(50).cloned().collect(),
            popular_tags: HashMap::new(),
        };

        // Count by category and user
        for record in storage.records.iter() {
            *analytics.records_by_category.entry(record.1.category.clone()).or_insert(0) += 1;
            *analytics.records_by_user.entry(record.1.created_by.to_string()).or_insert(0) += 1;
        }

        // Count tags
        let mut tag_counts: HashMap<String, u64> = HashMap::new();
        for record in storage.records.iter() {
            for tag in &record.1.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        // Get top 20 tags
        let mut tag_vec: Vec<(String, u64)> = tag_counts.into_iter().collect();
        tag_vec.sort_by(|a, b| b.1.cmp(&a.1));
        analytics.popular_tags = tag_vec.into_iter().take(20).collect();

        storage.stats_cache.set(analytics);
    });
}

fn update_category_index(storage: &mut StorageRef, category: &str, record_id: &str) {
    let mut category_records = storage.category_index.get(category).unwrap_or_default();
    if !category_records.contains(&record_id.to_string()) {
        category_records.push(record_id.to_string());
        storage.category_index.insert(category.to_string(), category_records);
    }
}

fn update_tag_index(storage: &mut StorageRef, tags: &[String], record_id: &str) {
    for tag in tags {
        let mut tag_records = storage.tag_index.get(tag).unwrap_or_default();
        if !tag_records.contains(&record_id.to_string()) {
            tag_records.push(record_id.to_string());
            storage.tag_index.insert(tag.clone(), tag_records);
        }
    }
}

fn update_user_index(storage: &mut StorageRef, user: Principal, record_id: &str) {
    let mut user_records = storage.user_records_index.get(&user).unwrap_or_default();
    if !user_records.contains(&record_id.to_string()) {
        user_records.push(record_id.to_string());
        storage.user_records_index.insert(user, user_records);
    }
}

fn remove_from_category_index(storage: &mut StorageRef, category: &str, record_id: &str) {
    if let Some(mut category_records) = storage.category_index.get(category) {
        category_records.retain(|id| id != record_id);
        storage.category_index.insert(category.to_string(), category_records);
    }
}

fn remove_from_tag_index(storage: &mut StorageRef, tags: &[String], record_id: &str) {
    for tag in tags {
        if let Some(mut tag_records) = storage.tag_index.get(tag) {
            tag_records.retain(|id| id != record_id);
            storage.tag_index.insert(tag.clone(), tag_records);
        }
    }
}

fn remove_from_user_index(storage: &mut StorageRef, user: Principal, record_id: &str) {
    if let Some(mut user_records) = storage.user_records_index.get(&user) {
        user_records.retain(|id| id != record_id);
        storage.user_records_index.insert(user, user_records);
    }
}

type StorageRef = std::cell::RefMut<'static, crate::StorageState>;