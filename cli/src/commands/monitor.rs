//! Monitoring dashboard for Icarus SDK
//!
//! Provides real-time monitoring of:
//! - Canister health and status
//! - Tool usage metrics
//! - Bridge performance
//! - Session activity

use anyhow::Result;
use candid::Principal;
use clap::Args;
use std::time::{Duration, Instant};
use tokio::time::interval;

/// Monitor canister and bridge performance
#[derive(Args)]
pub struct MonitorCommand {
    /// Canister ID to monitor
    #[arg(long)]
    pub canister_id: Option<String>,

    /// Refresh interval in seconds
    #[arg(long, default_value = "5")]
    pub interval: u64,

    /// Show detailed metrics
    #[arg(long)]
    pub detailed: bool,

    /// Output format (table, json)
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Monitor specific metrics only
    #[arg(long, value_delimiter = ',')]
    pub metrics: Vec<String>,
}

/// Canister health metrics
#[derive(Debug, Clone)]
pub struct CanisterHealth {
    pub status: String,
    pub cycles: Option<u64>,
    pub memory_usage: Option<u64>,
    pub response_time: Duration,
    pub last_updated: Instant,
}

/// Tool usage statistics
#[derive(Debug, Clone)]
pub struct ToolUsage {
    pub name: String,
    pub call_count: u64,
    pub avg_response_time: Duration,
    pub success_rate: f64,
    pub last_called: Option<Instant>,
}

/// Bridge performance metrics
#[derive(Debug, Clone)]
pub struct BridgeMetrics {
    pub uptime: Duration,
    pub total_requests: u64,
    pub active_sessions: u64,
    pub avg_response_time: Duration,
    pub error_rate: f64,
}

/// Monitoring dashboard
pub struct MonitoringDashboard {
    canister_id: Option<Principal>,
    refresh_interval: Duration,
    detailed: bool,
    format: String,
    start_time: Instant,
    metrics_filter: Vec<String>,
}

impl MonitoringDashboard {
    pub fn new(
        canister_id: Option<String>,
        interval: u64,
        detailed: bool,
        format: String,
        metrics: Vec<String>,
    ) -> Result<Self> {
        let canister_principal = if let Some(id) = canister_id {
            Some(Principal::from_text(&id)?)
        } else {
            None
        };

        Ok(Self {
            canister_id: canister_principal,
            refresh_interval: Duration::from_secs(interval),
            detailed,
            format,
            start_time: Instant::now(),
            metrics_filter: metrics,
        })
    }

    /// Start the monitoring dashboard
    pub async fn start(&self) -> Result<()> {
        println!("🔍 Starting Icarus SDK Monitoring Dashboard");
        println!("📊 Refresh interval: {} seconds", self.refresh_interval.as_secs());

        if let Some(canister_id) = &self.canister_id {
            println!("🎯 Monitoring canister: {}", canister_id);
        } else {
            println!("🌐 Monitoring all available canisters");
        }

        let mut interval_timer = interval(self.refresh_interval);

        loop {
            interval_timer.tick().await;

            match self.collect_metrics().await {
                Ok(metrics) => self.display_metrics(&metrics).await?,
                Err(e) => eprintln!("❌ Error collecting metrics: {}", e),
            }
        }
    }

    /// Collect all monitoring metrics
    async fn collect_metrics(&self) -> Result<MonitoringData> {
        let start = Instant::now();

        // Collect canister health
        let canister_health = if let Some(canister_id) = &self.canister_id {
            Some(self.collect_canister_health(*canister_id).await?)
        } else {
            None
        };

        // Collect tool usage stats
        let tool_usage = self.collect_tool_usage().await?;

        // Collect bridge metrics
        let bridge_metrics = self.collect_bridge_metrics().await?;

        Ok(MonitoringData {
            canister_health,
            tool_usage,
            bridge_metrics,
            collection_time: start.elapsed(),
            timestamp: Instant::now(),
        })
    }

    /// Collect canister health metrics
    async fn collect_canister_health(&self, _canister_id: Principal) -> Result<CanisterHealth> {
        let start = Instant::now();

        // TODO: Implement actual canister health checks
        // For now, return mock data
        Ok(CanisterHealth {
            status: "Running".to_string(),
            cycles: Some(1_000_000_000_000), // 1T cycles
            memory_usage: Some(50 * 1024 * 1024), // 50MB
            response_time: start.elapsed(),
            last_updated: Instant::now(),
        })
    }

    /// Collect tool usage statistics
    async fn collect_tool_usage(&self) -> Result<Vec<ToolUsage>> {
        // TODO: Implement actual tool usage collection
        // For now, return mock data
        Ok(vec![
            ToolUsage {
                name: "get_item".to_string(),
                call_count: 45,
                avg_response_time: Duration::from_millis(150),
                success_rate: 0.98,
                last_called: Some(Instant::now() - Duration::from_secs(30)),
            },
            ToolUsage {
                name: "create_item".to_string(),
                call_count: 23,
                avg_response_time: Duration::from_millis(280),
                success_rate: 0.95,
                last_called: Some(Instant::now() - Duration::from_secs(120)),
            },
        ])
    }

    /// Collect bridge performance metrics
    async fn collect_bridge_metrics(&self) -> Result<BridgeMetrics> {
        Ok(BridgeMetrics {
            uptime: self.start_time.elapsed(),
            total_requests: 68,
            active_sessions: 2,
            avg_response_time: Duration::from_millis(205),
            error_rate: 0.02,
        })
    }

    /// Display metrics in the specified format
    async fn display_metrics(&self, data: &MonitoringData) -> Result<()> {
        // Clear screen for better viewing
        print!("\x1B[2J\x1B[1;1H");

        match self.format.as_str() {
            "json" => self.display_json(data).await?,
            _ => self.display_table(data).await?,
        }

        Ok(())
    }

    /// Display metrics in table format
    async fn display_table(&self, data: &MonitoringData) -> Result<()> {
        println!("🔍 Icarus SDK Monitoring Dashboard");
        println!("⏰ Last updated: {} ms ago", data.collection_time.as_millis());
        println!("{}", "─".repeat(60));

        // Canister Health Section
        if let Some(health) = &data.canister_health {
            println!("🏥 CANISTER HEALTH");
            println!("  Status:       {}", health.status);
            if let Some(cycles) = health.cycles {
                println!("  Cycles:       {:.2}T", cycles as f64 / 1_000_000_000_000.0);
            }
            if let Some(memory) = health.memory_usage {
                println!("  Memory:       {:.1}MB", memory as f64 / (1024.0 * 1024.0));
            }
            println!("  Response:     {}ms", health.response_time.as_millis());
            println!();
        }

        // Bridge Metrics Section
        println!("🌉 BRIDGE PERFORMANCE");
        let metrics = &data.bridge_metrics;
        println!("  Uptime:       {:.1}h", metrics.uptime.as_secs() as f64 / 3600.0);
        println!("  Requests:     {}", metrics.total_requests);
        println!("  Sessions:     {}", metrics.active_sessions);
        println!("  Avg Response: {}ms", metrics.avg_response_time.as_millis());
        println!("  Error Rate:   {:.1}%", metrics.error_rate * 100.0);
        println!();

        // Tool Usage Section
        if !data.tool_usage.is_empty() {
            println!("🛠️  TOOL USAGE");
            println!("  {:<20} {:<8} {:<10} {:<8} {:<12}", "Tool", "Calls", "Avg Time", "Success", "Last Called");
            println!("  {:<20} {:<8} {:<10} {:<8} {:<12}", "────", "─────", "────────", "───────", "───────────");

            for tool in &data.tool_usage {
                let last_called = if let Some(last) = tool.last_called {
                    let secs = last.elapsed().as_secs();
                    if secs < 60 {
                        format!("{}s ago", secs)
                    } else if secs < 3600 {
                        format!("{}m ago", secs / 60)
                    } else {
                        format!("{}h ago", secs / 3600)
                    }
                } else {
                    "Never".to_string()
                };

                println!(
                    "  {:<20} {:<8} {:<10} {:<8} {:<12}",
                    tool.name,
                    tool.call_count,
                    format!("{}ms", tool.avg_response_time.as_millis()),
                    format!("{:.1}%", tool.success_rate * 100.0),
                    last_called
                );
            }
            println!();
        }

        if self.detailed {
            println!("📊 DETAILED METRICS");
            println!("  Collection time: {}ms", data.collection_time.as_millis());
            println!("  Dashboard uptime: {:.1}h", self.start_time.elapsed().as_secs() as f64 / 3600.0);
        }

        Ok(())
    }

    /// Display metrics in JSON format
    async fn display_json(&self, data: &MonitoringData) -> Result<()> {
        let json_data = serde_json::json!({
            "timestamp": data.timestamp.elapsed().as_secs(),
            "collection_time_ms": data.collection_time.as_millis(),
            "canister_health": data.canister_health.as_ref().map(|h| serde_json::json!({
                "status": h.status,
                "cycles": h.cycles,
                "memory_usage": h.memory_usage,
                "response_time_ms": h.response_time.as_millis()
            })),
            "bridge_metrics": {
                "uptime_seconds": data.bridge_metrics.uptime.as_secs(),
                "total_requests": data.bridge_metrics.total_requests,
                "active_sessions": data.bridge_metrics.active_sessions,
                "avg_response_time_ms": data.bridge_metrics.avg_response_time.as_millis(),
                "error_rate": data.bridge_metrics.error_rate
            },
            "tool_usage": data.tool_usage.iter().map(|t| serde_json::json!({
                "name": t.name,
                "call_count": t.call_count,
                "avg_response_time_ms": t.avg_response_time.as_millis(),
                "success_rate": t.success_rate,
                "last_called_seconds_ago": t.last_called.map(|l| l.elapsed().as_secs())
            })).collect::<Vec<_>>()
        });

        println!("{}", serde_json::to_string_pretty(&json_data)?);
        Ok(())
    }
}

/// Container for all monitoring data
#[derive(Debug)]
struct MonitoringData {
    canister_health: Option<CanisterHealth>,
    tool_usage: Vec<ToolUsage>,
    bridge_metrics: BridgeMetrics,
    collection_time: Duration,
    timestamp: Instant,
}

impl MonitorCommand {
    pub async fn run(&self) -> Result<()> {
        let dashboard = MonitoringDashboard::new(
            self.canister_id.clone(),
            self.interval,
            self.detailed,
            self.format.clone(),
            self.metrics.clone(),
        )?;

        dashboard.start().await
    }
}