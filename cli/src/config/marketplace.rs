use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceMetadata {
    pub name: String,
    pub description: String,
    pub categories: Vec<String>,
    pub price_icp: f64,
    pub author_revenue_share: u8,
    pub minimum_cycles: u64,
    pub version: String,
    pub screenshots: Vec<String>,
    pub readme: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub logo_url: Option<String>,
    pub images: Vec<String>,
    pub documentation_url: Option<String>,
    pub developer_address: Option<String>, // Principal as string
}

impl Default for MarketplaceMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            categories: vec!["tools".to_string()],
            price_icp: 1.0,
            author_revenue_share: 80,
            minimum_cycles: 1_000_000_000_000, // 1T cycles
            version: "1.0.0".to_string(),
            screenshots: Vec::new(),
            readme: Some("README.md".to_string()),
            license: Some("MIT".to_string()),
            repository: None,
            keywords: Vec::new(),
            logo_url: None,
            images: Vec::new(),
            documentation_url: None,
            developer_address: None,
        }
    }
}

impl MarketplaceMetadata {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Name cannot be empty");
        }
        if self.description.is_empty() {
            anyhow::bail!("Description cannot be empty");
        }
        if self.description.len() < 20 {
            anyhow::bail!("Description must be at least 20 characters");
        }
        if self.categories.is_empty() {
            anyhow::bail!("At least one category must be specified");
        }
        if self.price_icp < 0.0 {
            anyhow::bail!("Price cannot be negative");
        }
        if self.author_revenue_share > 100 {
            anyhow::bail!("Revenue share cannot exceed 100%");
        }
        if self.minimum_cycles < 100_000_000_000 {
            anyhow::bail!("Minimum cycles must be at least 100B");
        }
        Ok(())
    }
}

// Categories available in the marketplace
pub const MARKETPLACE_CATEGORIES: &[&str] = &[
    "productivity",
    "ai-tools",
    "development",
    "data-analysis",
    "communication",
    "automation",
    "monitoring",
    "security",
    "finance",
    "entertainment",
    "education",
    "utilities",
];
