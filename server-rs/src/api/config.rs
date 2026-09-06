//! Configuration for the public API server

use std::env;

use serde::{Deserialize, Serialize};

/// OpenID Connect configuration — see `docs/developer_guide/AUTHENTICATION.md`.
///
/// All fields are read from environment variables (`OIDC_*`), same convention as
/// [`ApiServerConfig`]. When `enabled` is `false` (the default), the whole
/// authentication layer is bypassed and every request is treated as an unrestricted
/// admin — identical to the server's behavior before this feature existed, so upgrading
/// an existing deployment without setting `OIDC_ENABLED=true` changes nothing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub enabled: bool,
    pub issuer_url: String,
    pub client_id: String,
    /// `None` — public client, PKCE only (no client secret configured in the IdP).
    pub client_secret: Option<String>,
    /// Base URL the *browser* uses to reach this server (may differ from the URL this
    /// process uses to reach the IdP, e.g. behind Docker Compose or a reverse proxy).
    /// `/auth/callback` is appended to build the OIDC redirect URI.
    pub redirect_base_url: String,
    /// Dotted path into the ID token claims used to determine admin membership, e.g.
    /// `realm_access.roles` (Keycloak) or `groups` (many generic OIDC/Azure AD setups).
    pub admin_role_claim_path: String,
    /// Values that, if present at `admin_role_claim_path`, grant the admin role.
    pub admin_role_values: Vec<String>,
    /// Claim whose value identifies the user for matching against a host's `owners`
    /// list (case-insensitively), e.g. `preferred_username` or `email`.
    pub identity_claim: String,
    pub session_ttl_secs: u64,
    pub cookie_name: String,
    /// Whether the session cookie is marked `Secure` (HTTPS only). Only disable for
    /// local HTTP development.
    pub cookie_secure: bool,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("OIDC_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            issuer_url: env::var("OIDC_ISSUER_URL").unwrap_or_default(),
            client_id: env::var("OIDC_CLIENT_ID").unwrap_or_default(),
            client_secret: env::var("OIDC_CLIENT_SECRET")
                .ok()
                .filter(|s| !s.is_empty()),
            redirect_base_url: env::var("OIDC_REDIRECT_BASE_URL").unwrap_or_default(),
            admin_role_claim_path: env::var("OIDC_ADMIN_ROLE_CLAIM_PATH")
                .unwrap_or_else(|_| "realm_access.roles".to_string()),
            admin_role_values: env::var("OIDC_ADMIN_ROLE_VALUES")
                .ok()
                .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|| vec!["admin".to_string()]),
            identity_claim: env::var("OIDC_IDENTITY_CLAIM")
                .unwrap_or_else(|_| "preferred_username".to_string()),
            session_ttl_secs: env::var("OIDC_SESSION_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(28800),
            cookie_name: env::var("OIDC_COOKIE_NAME")
                .unwrap_or_else(|_| "woodstock_session".to_string()),
            cookie_secure: env::var("OIDC_COOKIE_SECURE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
        }
    }
}

impl OidcConfig {
    /// Redirect URI registered with the IdP (`{redirect_base_url}/auth/callback`).
    #[must_use]
    pub fn redirect_uri(&self) -> String {
        format!(
            "{}/auth/callback",
            self.redirect_base_url.trim_end_matches('/')
        )
    }

    /// Where the IdP should send the browser back to once it has ended its own SSO session
    /// (see `auth::oidc::OidcClient::end_session_url`). Must be registered on the IdP
    /// client as a valid post-logout redirect URI — Keycloak exposes this as its own
    /// "Valid post logout redirect URIs" field (separate from "Valid Redirect URIs",
    /// defaulting to `+`, i.e. "reuse Valid Redirect URIs" — which won't match this root
    /// URL if that list only has the exact `{redirect_uri()}` path). Set the field
    /// explicitly rather than adding this URL to "Valid Redirect URIs" too.
    #[must_use]
    pub fn post_logout_redirect_uri(&self) -> String {
        format!("{}/", self.redirect_base_url.trim_end_matches('/'))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    /// Hostname to bind to
    pub hostname: String,

    /// Port for the public API server (default: 3000)
    pub port: u16,

    /// Path to static files to serve (e.g. frontend)
    pub static_path: Option<String>,

    /// OpenID Connect authentication settings
    pub oidc: OidcConfig,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            hostname: env::var("MANAGEMENT_API_LISTEN").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("MANAGEMENT_API_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            static_path: env::var("STATIC_PATH").ok(),
            oidc: OidcConfig::default(),
        }
    }
}

impl ApiServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Get API bind address
    pub fn api_address(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }
}
