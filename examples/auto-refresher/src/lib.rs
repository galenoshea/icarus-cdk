//! Auto-Refresher Example
//!
//! Demonstrates combining HTTP outcalls with timers for autonomous data fetching.
//! This canister periodically fetches external data and caches it for fast access.

use icarus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Data structure for cached prices
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: u64,
}

// Store prices and timer IDs in stable memory
stable_storage! {
    PRICE_CACHE: StableBTreeMap<String, PriceData, Memory> = memory_id!(0);
    TIMER_IDS: StableCell<Vec<u64>, Memory> = memory_id!(1);
}

#[icarus_module]
mod refresher {
    use super::*;
    use ic_cdk_timers::TimerId;
    use std::cell::RefCell;

    thread_local! {
        // Keep track of active timer IDs locally
        static ACTIVE_TIMERS: RefCell<Vec<TimerId>> = RefCell::new(Vec::new());
    }

    /// Initialize the auto-refresher with periodic data fetching
    #[init]
    fn init() {
        // Start a timer to refresh data every 5 minutes
        start_auto_refresh();
        ic_cdk::print("Auto-refresher initialized with 5-minute interval");
    }

    /// Start automatic data refreshing
    #[update]
    #[icarus_tool("Start automatic price updates every 5 minutes")]
    pub fn start_auto_refresh() -> Result<String, String> {
        // Cancel any existing timers first
        stop_auto_refresh();

        // Schedule periodic price updates (every 5 minutes)
        let timer = timers::schedule_periodic(300, "price-refresh", || {
            ic_cdk::spawn(async {
                let _ = fetch_and_cache_prices().await;
            });
        })
        .map_err(|e| format!("Failed to schedule timer: {}", e))?;

        // Store timer ID
        ACTIVE_TIMERS.with(|t| t.borrow_mut().push(timer));

        Ok("Auto-refresh started: fetching prices every 5 minutes".to_string())
    }

    /// Stop automatic data refreshing
    #[update]
    #[icarus_tool("Stop automatic price updates")]
    pub fn stop_auto_refresh() -> Result<String, String> {
        let mut count = 0;
        ACTIVE_TIMERS.with(|t| {
            let timers = t.borrow();
            for timer in timers.iter() {
                let _ = timers::cancel_timer(*timer);
                count += 1;
            }
            t.borrow_mut().clear();
        });

        Ok(format!("Stopped {} active timers", count))
    }

    /// Manually fetch and cache prices
    #[update]
    #[icarus_tool("Manually trigger a price update")]
    pub async fn fetch_and_cache_prices() -> Result<Vec<PriceData>, String> {
        let symbols = vec!["BTC", "ETH", "ICP"];
        let mut results = Vec::new();

        for symbol in symbols {
            // In a real application, you'd fetch from a price API
            // For demo purposes, we'll simulate with a mock endpoint
            let url = format!(
                "https://api.coinbase.com/v2/exchange-rates?currency={}",
                symbol
            );

            match http::get(&url).await {
                Ok(response) => {
                    // Parse the response (simplified for demo)
                    let price_data = PriceData {
                        symbol: symbol.to_string(),
                        price: extract_price(&response, symbol),
                        timestamp: ic_cdk::api::time(),
                    };

                    // Cache the price
                    PRICE_CACHE.with(|cache| {
                        cache
                            .borrow_mut()
                            .insert(symbol.to_string(), price_data.clone())
                    });

                    results.push(price_data);
                }
                Err(e) => {
                    ic_cdk::print(&format!("Failed to fetch {}: {}", symbol, e));
                    // Continue with other symbols even if one fails
                }
            }
        }

        Ok(results)
    }

    /// Get cached price for a symbol
    #[query]
    #[icarus_tool("Get the latest cached price for a cryptocurrency")]
    pub fn get_cached_price(symbol: String) -> Result<PriceData, String> {
        PRICE_CACHE.with(|cache| {
            cache
                .borrow()
                .get(&symbol)
                .ok_or_else(|| format!("No cached price for {}", symbol))
        })
    }

    /// Get all cached prices
    #[query]
    #[icarus_tool("Get all cached cryptocurrency prices")]
    pub fn get_all_prices() -> Vec<PriceData> {
        PRICE_CACHE.with(|cache| {
            cache
                .borrow()
                .iter()
                .map(|(_, price)| price.clone())
                .collect()
        })
    }

    /// Schedule a one-time data refresh
    #[update]
    #[icarus_tool("Schedule a one-time price update after specified seconds")]
    pub fn schedule_refresh(delay_seconds: u64) -> Result<String, String> {
        let timer = timers::schedule_once(delay_seconds, "one-time-refresh", || {
            ic_cdk::spawn(async {
                let _ = fetch_and_cache_prices().await;
            });
        })
        .map_err(|e| format!("Failed to schedule refresh: {}", e))?;

        ACTIVE_TIMERS.with(|t| t.borrow_mut().push(timer));

        Ok(format!(
            "Scheduled one-time refresh in {} seconds",
            delay_seconds
        ))
    }

    /// Get status of active timers
    #[query]
    #[icarus_tool("Get information about active timers")]
    pub fn get_timer_status() -> String {
        let timer_count = timers::active_timer_count();
        let active_timers = timers::list_active_timers();

        let mut status = format!("Active timers: {}\n", timer_count);
        for timer in active_timers {
            status.push_str(&format!(
                "- {} ({}): every {:?} seconds\n",
                timer.name,
                match timer.timer_type {
                    TimerType::Once => "one-time",
                    TimerType::Periodic => "periodic",
                },
                timer.interval_secs
            ));
        }

        status
    }

    /// Clear all cached data
    #[update]
    #[icarus_tool("Clear all cached price data")]
    pub fn clear_cache() -> Result<String, String> {
        PRICE_CACHE.with(|cache| {
            let count = cache.borrow().len();
            cache.borrow_mut().clear();
            Ok(format!("Cleared {} cached prices", count))
        })
    }

    // Helper function to extract price from response (simplified)
    fn extract_price(response: &str, symbol: &str) -> f64 {
        // In a real app, you'd parse JSON properly
        // This is simplified for the example
        match symbol {
            "BTC" => 45000.0,
            "ETH" => 2800.0,
            "ICP" => 12.5,
            _ => 0.0,
        }
    }
}

// Re-export the icarus metadata function
pub use refresher::icarus_metadata;