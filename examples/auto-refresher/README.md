# Auto-Refresher Example

This example demonstrates how to combine HTTP outcalls with timers to create an autonomous data-fetching canister.

## Features

- **Periodic Data Fetching**: Automatically fetches cryptocurrency prices every 5 minutes
- **HTTP Outcalls**: Retrieves data from external APIs
- **Timer Management**: Start, stop, and monitor active timers
- **Stable Storage**: Caches fetched data in persistent memory
- **Manual Controls**: Trigger updates on-demand or schedule one-time refreshes

## Quick Start

```bash
# Deploy locally
dfx start --clean
dfx deploy

# Start auto-refresh (fetches prices every 5 minutes)
dfx canister call auto_refresher start_auto_refresh

# Get cached prices
dfx canister call auto_refresher get_all_prices

# Get specific price
dfx canister call auto_refresher get_cached_price '("BTC")'

# Check timer status
dfx canister call auto_refresher get_timer_status

# Schedule one-time refresh in 30 seconds
dfx canister call auto_refresher schedule_refresh '(30)'

# Stop auto-refresh
dfx canister call auto_refresher stop_auto_refresh
```

## Use with Claude Desktop

```bash
# Build and deploy
icarus build
icarus deploy

# Add to Claude Desktop
icarus bridge add <canister-id> --name "Crypto Price Tracker"

# Now Claude can:
# - Monitor cryptocurrency prices
# - Schedule price updates
# - Access historical price data
```

## How It Works

1. **Initialization**: On canister creation, starts a timer to fetch prices every 5 minutes
2. **Data Fetching**: Uses HTTP outcalls to retrieve price data from external APIs
3. **Caching**: Stores fetched data in stable memory for fast access
4. **Timer Management**: Provides tools to control and monitor active timers
5. **Query Interface**: Allows instant access to cached data without network calls

## Key Concepts Demonstrated

- **`timers::schedule_periodic()`**: Creates recurring tasks
- **`timers::schedule_once()`**: Creates one-time delayed tasks
- **`http::get()`**: Fetches data from external APIs
- **Stable Storage**: Persists data across canister upgrades
- **Async Operations**: Combines timers with async HTTP calls

## Customization

You can modify this example to:
- Fetch different types of data (weather, news, etc.)
- Adjust refresh intervals
- Add data processing or analysis
- Implement alerts when prices change
- Store historical data for trend analysis