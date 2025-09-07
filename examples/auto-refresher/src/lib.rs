//! Auto-Refresher Example
//!
//! Demonstrates combining HTTP outcalls with timers for autonomous data fetching.

use candid::CandidType;
use ic_cdk::api;
use ic_cdk_macros::{query, update};
use icarus::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct PriceData {
    symbol: String,
    price: f64,
    timestamp: u64,
}

thread_local! {
    static PRICE_CACHE: RefCell<Vec<PriceData>> = RefCell::new(Vec::new());
}

#[icarus_module]
mod tools {
    use super::*;

    #[update]
    #[icarus_tool("Fetch cryptocurrency prices")]
    pub async fn fetch_prices() -> Vec<PriceData> {
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
    #[icarus_tool("Get cached prices")]
    pub fn get_cached_prices() -> Vec<PriceData> {
        PRICE_CACHE.with(|cache| cache.borrow().clone())
    }

    #[update]
    #[icarus_tool("Clear price cache")]
    pub fn clear_cache() -> String {
        PRICE_CACHE.with(|cache| {
            let count = cache.borrow().len();
            cache.borrow_mut().clear();
            format!("Cleared {} cached prices", count)
        })
    }

    #[update]
    #[icarus_tool("Schedule a price refresh")]
    pub fn schedule_refresh(seconds: u64) -> String {
        // In a real app, would use timers::schedule_once
        format!("Would schedule refresh in {} seconds", seconds)
    }
}

// Export the Candid interface for the canister
ic_cdk::export_candid!();
