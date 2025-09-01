//! OAuth2 endpoints for Claude Desktop integration
//!
//! Provides OAuth2 authorization and token endpoints for secure
//! authentication between Claude Desktop and the MCP bridge.

use anyhow::{Context, Result};
use candid::Principal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::Filter;

/// OAuth2 token response
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// OAuth2 authorization request
#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub scope: Option<String>,
}

/// OAuth2 token request
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub refresh_token: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
}

/// Authorization code data
#[derive(Debug, Clone)]
struct AuthCodeData {
    pub principal: Principal,
    pub client_id: String,
    pub redirect_uri: String,
    pub expires_at: u64,
    pub canister_id: String,
}

/// OAuth2 server
pub struct OAuth2Server {
    auth_codes: Arc<Mutex<HashMap<String, AuthCodeData>>>,
    canister_id: String,
    port: u16,
}

impl OAuth2Server {
    /// Create a new OAuth2 server
    pub fn new(canister_id: String, port: u16, _secret_key: Option<String>) -> Self {
        Self {
            auth_codes: Arc::new(Mutex::new(HashMap::new())),
            canister_id,
            port,
        }
    }

    /// Start the OAuth2 server
    pub async fn start(self: Arc<Self>) -> Result<()> {
        let oauth2_server = self.clone();

        // Authorization endpoint
        let authorize = warp::path("oauth")
            .and(warp::path("authorize"))
            .and(warp::query::<AuthorizeRequest>())
            .and_then({
                let server = oauth2_server.clone();
                move |req: AuthorizeRequest| {
                    let server = server.clone();
                    async move {
                        server
                            .handle_authorize(req)
                            .await
                            .map_err(|e| warp::reject::custom(OAuth2Error::from(e)))
                    }
                }
            });

        // Token endpoint
        let token = warp::path("oauth")
            .and(warp::path("token"))
            .and(warp::body::form::<TokenRequest>())
            .and_then({
                let server = oauth2_server.clone();
                move |req: TokenRequest| {
                    let server = server.clone();
                    async move {
                        server
                            .handle_token(req)
                            .await
                            .map_err(|e| warp::reject::custom(OAuth2Error::from(e)))
                    }
                }
            });

        // Combine routes
        let routes = authorize.or(token);

        println!("OAuth2 server listening on http://localhost:{}", self.port);

        warp::serve(routes).run(([127, 0, 0, 1], self.port)).await;

        Ok(())
    }

    /// Handle authorization request
    async fn handle_authorize(&self, req: AuthorizeRequest) -> Result<impl warp::Reply> {
        // Validate request
        if req.response_type != "code" {
            anyhow::bail!("Unsupported response type");
        }

        // Get the authenticated principal (in production, this would involve II auth)
        // For now, we'll use a test principal
        let principal = self.get_authenticated_principal().await?;

        // Generate authorization code
        let auth_code = Uuid::new_v4().to_string();

        // Store auth code data
        let auth_data = AuthCodeData {
            principal,
            client_id: req.client_id,
            redirect_uri: req.redirect_uri.clone(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs()
                + 300, // 5 minutes
            canister_id: self.canister_id.clone(),
        };

        self.auth_codes
            .lock()
            .await
            .insert(auth_code.clone(), auth_data);

        // Build redirect URL
        let mut redirect_url = req.redirect_uri;
        redirect_url.push_str(if redirect_url.contains('?') { "&" } else { "?" });
        redirect_url.push_str(&format!("code={}", auth_code));

        if let Some(state) = req.state {
            redirect_url.push_str(&format!("&state={}", state));
        }

        // Return redirect response
        Ok(warp::redirect::temporary(
            redirect_url
                .parse::<warp::http::Uri>()
                .context("Invalid redirect URI")?,
        ))
    }

    /// Handle token request
    async fn handle_token(&self, req: TokenRequest) -> Result<impl warp::Reply> {
        let response = match req.grant_type.as_str() {
            "authorization_code" => {
                // Exchange authorization code for tokens
                let code = req.code.context("Missing authorization code")?;

                // Retrieve and validate auth code
                let auth_data = self
                    .auth_codes
                    .lock()
                    .await
                    .remove(&code)
                    .context("Invalid or expired authorization code")?;

                // Check if code is expired
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();

                if auth_data.expires_at < now {
                    anyhow::bail!("Authorization code expired");
                }

                // Validate client
                if auth_data.client_id != req.client_id {
                    anyhow::bail!("Client ID mismatch");
                }

                // Generate tokens (simplified without JWT)
                OAuth2TokenResponse {
                    access_token: format!("token_{}", Uuid::new_v4()),
                    token_type: "Bearer".to_string(),
                    expires_in: 3600,
                    refresh_token: Some(format!("refresh_{}", Uuid::new_v4())),
                    scope: Some("read write".to_string()),
                }
            }
            "refresh_token" => {
                // Refresh access token (simplified without JWT)
                let _refresh_token = req.refresh_token.context("Missing refresh token")?;

                // Generate new tokens
                OAuth2TokenResponse {
                    access_token: format!("token_{}", Uuid::new_v4()),
                    token_type: "Bearer".to_string(),
                    expires_in: 3600,
                    refresh_token: Some(format!("refresh_{}", Uuid::new_v4())),
                    scope: Some("read write".to_string()),
                }
            }
            _ => anyhow::bail!("Unsupported grant type"),
        };

        Ok(warp::reply::json(&response))
    }

    /// Get authenticated principal from stored session (simplified)
    async fn get_authenticated_principal(&self) -> Result<Principal> {
        // For now, return anonymous principal since auth is removed
        Ok(Principal::anonymous())
    }
}

/// OAuth2 error for warp rejection
#[derive(Debug)]
struct OAuth2Error {
    message: String,
}

impl From<anyhow::Error> for OAuth2Error {
    fn from(err: anyhow::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl warp::reject::Reject for OAuth2Error {}

/// Generate Claude Desktop configuration for OAuth2
pub fn generate_claude_config(canister_id: &str, oauth_port: u16) -> serde_json::Value {
    serde_json::json!({
        "mcpServers": {
            format!("icarus-{}", canister_id): {
                "command": "icarus",
                "args": [
                    "bridge",
                    "start",
                    "--canister-id", canister_id,
                    "--oauth-port", oauth_port.to_string()
                ],
                "env": {
                    "OAUTH_ENDPOINT": format!("http://localhost:{}/oauth", oauth_port),
                    "CANISTER_ID": canister_id,
                }
            }
        }
    })
}
