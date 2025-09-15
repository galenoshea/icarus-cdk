# API Gateway Template

A production-ready template for creating API integrations and external service connectors with caching, rate limiting, and monitoring.

## Features

### 🌐 HTTP Client
- Support for GET, POST, PUT, DELETE, PATCH methods
- Automatic request/response handling
- Configurable timeouts and retries
- Custom header management

### 🔐 Authentication
- API Key authentication
- Bearer token support
- Basic authentication
- Custom auth header injection

### ⚡ Performance
- Response caching with TTL
- Rate limiting protection
- Request batching
- Connection pooling

### 📊 Monitoring
- Request/response logging
- Error tracking and metrics
- Performance analytics
- Health checks

## Quick Start

1. **Create from Template**:
   ```bash
   icarus new my-api-gateway --template api-gateway
   cd my-api-gateway
   ```

2. **Deploy**:
   ```bash
   dfx start --background
   icarus build
   icarus deploy --network local
   ```

## Available Tools

- `call_api` - Call external API endpoint
- `register_endpoint` - Register new API endpoint
- `get_endpoint` - Get endpoint configuration
- `update_endpoint` - Update endpoint settings
- `get_cache_stats` - View caching statistics

## Usage Examples

### Register an API Endpoint
```javascript
Human: Register a new weather API endpoint at https://api.weather.com/v1/current with API key authentication

Claude: I'll register that weather API endpoint for you.
[Uses register_endpoint tool]
✅ Weather API endpoint registered with ID: weather-123
```

### Make API Calls
```javascript
Human: Call the weather API for New York

Claude: I'll get the weather data for New York.
[Uses call_api tool with location parameter]
🌤️ Current weather in New York: 72°F, partly cloudy
```

This is a simplified template. The full implementation would include comprehensive HTTP client functionality, caching, rate limiting, and monitoring features.