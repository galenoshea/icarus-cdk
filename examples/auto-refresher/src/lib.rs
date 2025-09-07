//! Auto-Refresher Example
//!
//! Demonstrates combining HTTP outcalls with timers for autonomous data fetching.

use candid::CandidType;
use ic_cdk::api;
use ic_cdk_macros::{init, query, update};
use icarus_canister::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: u64,
}

thread_local! {
    static PRICE_CACHE: RefCell<Vec<PriceData>> = RefCell::new(Vec::new());
}

#[init]
fn init() {
    ic_cdk::print("Auto-refresher initialized");
}

#[update]
async fn fetch_prices() -> Vec<PriceData> {
    let symbols = vec!["BTC", "ETH", "ICP"];
    let mut results = Vec::new();

    for symbol in symbols {
        // Simulate fetching (in real app, use http::get)
        let price_data = PriceData {
            symbol: symbol.to_string(),
            price: match symbol {
                "BTC" => 45000.0,
                "ETH" => 2800.0,
                "ICP" => 12.5,
                _ => 0.0,
            },
            timestamp: api::time(),
        };
        results.push(price_data.clone());

        // Cache the price
        PRICE_CACHE.with(|cache| {
            cache.borrow_mut().push(price_data);
        });
    }

    results
}

#[query]
fn get_cached_prices() -> Vec<PriceData> {
    PRICE_CACHE.with(|cache| cache.borrow().clone())
}

#[update]
fn clear_cache() -> String {
    PRICE_CACHE.with(|cache| {
        let count = cache.borrow().len();
        cache.borrow_mut().clear();
        format!("Cleared {} cached prices", count)
    })
}

#[update]
fn schedule_refresh(seconds: u64) -> String {
    // In a real app, would use timers::schedule_once
    format!("Would schedule refresh in {} seconds", seconds)
}

// Export metadata for MCP tools discovery
#[query]
fn icarus_metadata() -> String {
    r#"{
        "name": "auto-refresher",
        "description": "Example showing timers and HTTP outcalls",
        "tools": [
            {
                "name": "fetch_prices",
                "description": "Fetch cryptocurrency prices"
            },
            {
                "name": "get_cached_prices",
                "description": "Get cached prices"
            },
            {
                "name": "clear_cache",
                "description": "Clear price cache"
            }
        ]
    }"#
    .to_string()
}
