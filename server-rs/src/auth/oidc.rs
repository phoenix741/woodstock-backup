//! OpenID Connect client: discovery, PKCE, Authorization Code exchange, and ID token
//! validation — all delegated to the `openidconnect` crate. Nothing here parses or
//! verifies a token by hand: discovery fetches the IdP's JWKS, and
//! `id_token.claims(&verifier, &nonce)` is what performs the actual signature and
//! `iss`/`aud`/`exp`/`nonce` validation before any claim is trusted.

use std::collections::HashMap;
use std::str::FromStr;

use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreErrorResponseType,
    CoreGenderClaim, CoreIdToken, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm, CoreProviderMetadata, CoreRevocableToken, CoreRevocationErrorResponse,
    CoreTokenIntrospectionResponse, CoreTokenType,
};
use openidconnect::{
    AdditionalClaims, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, EndSessionUrl, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IdTokenFields, IssuerUrl, LogoutRequest, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    PostLogoutRedirectUrl, ProviderMetadataWithLogout, RedirectUrl, Scope, StandardErrorResponse,
    StandardTokenResponse, TokenResponse,
};
use tokio::sync::OnceCell;

/// Captures every ID token claim the OIDC Core spec doesn't already model as a typed
/// field — Keycloak's `realm_access`/`resource_access`, a flat `groups` array used by many
/// other IdPs, etc. This exists because `openidconnect::core::CoreIdTokenClaims` is fixed
/// to `EmptyAdditionalClaims`, which **silently discards any claim outside the standard
/// OIDC set** when the crate deserializes the ID token — including `realm_access`, the
/// claim `OIDC_ADMIN_ROLE_CLAIM_PATH` needs by default. Using this catch-all type as the
/// whole client stack's `AdditionalClaims` parameter (below) instead is what makes those
/// claims survive deserialization at all; nothing here parses or trusts them before
/// `id_token.claims(...)`'s own signature/`iss`/`aud`/`exp`/`nonce` validation has run.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct ExtraClaims(HashMap<String, serde_json::Value>);
impl AdditionalClaims for ExtraClaims {}

type WoodstockIdTokenFields = IdTokenFields<
    ExtraClaims,
    EmptyExtraTokenFields,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;
type WoodstockTokenResponse = StandardTokenResponse<WoodstockIdTokenFields, CoreTokenType>;

/// Same bundle of types as `openidconnect::core::CoreClient`, except parameterized with
/// [`ExtraClaims`] instead of `EmptyAdditionalClaims` so ID token claims outside the OIDC
/// Core spec aren't dropped (see [`ExtraClaims`]'s doc comment for why that matters here).
type WoodstockClient<
    HasAuthUrl = EndpointNotSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointNotSet,
    HasUserInfoUrl = EndpointNotSet,
> = Client<
    ExtraClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    WoodstockTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
    HasUserInfoUrl,
>;

/// [`WoodstockClient::from_provider_metadata`] fixes the auth endpoint as always-present
/// (`EndpointSet`) and the token/userinfo endpoints as discovered-but-not-statically-
/// guaranteed (`EndpointMaybeSet`) — this alias names that exact instantiation so it can
/// be stored in a field/`OnceCell` (the bare `WoodstockClient` alias defaults every
/// endpoint to `EndpointNotSet`, which does not match what discovery actually produces).
type DiscoveredCoreClient = WoodstockClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

use crate::api::config::OidcConfig;
use crate::api::ApiError;
use crate::auth::authz::{claim_grants_admin, resolve_claim_path};

/// What `/auth/login` hands to the caller: where to redirect the browser, plus the
/// per-flow secrets that must round-trip through the session store until `/auth/callback`.
pub struct AuthorizationRequest {
    pub authorize_url: String,
    pub csrf_token: String,
    pub nonce: String,
    pub pkce_verifier: String,
}

/// The caller's identity as resolved from an already-validated ID token.
pub struct AuthenticatedIdentity {
    pub subject: String,
    pub identity: String,
    pub is_admin: bool,
    /// The raw ID token (compact JWT), kept only to pass as `id_token_hint` to the IdP's
    /// end-session endpoint on logout (see [`OidcClient::end_session_url`]) — this crate's
    /// own type is reused as a carrier for that hint, not re-parsed or re-verified by hand.
    pub id_token: String,
}

pub struct OidcClient {
    config: OidcConfig,
    http_client: openidconnect::reqwest::Client,
    core_client: OnceCell<DiscoveredCoreClient>,
    /// [OpenID Connect RP-Initiated Logout](https://openid.net/specs/openid-connect-rpinitiated-1_0.html)
    /// is an optional extension: `Some(None)` once resolved means the IdP was checked and
    /// doesn't advertise one, in which case [`Self::end_session_url`] falls back to a purely
    /// local logout.
    logout_endpoint: OnceCell<Option<EndSessionUrl>>,
}

impl OidcClient {
    #[must_use]
    pub fn new(config: OidcConfig) -> Self {
        Self {
            config,
            // A dedicated client isolated to IdP calls (discovery, token exchange) —
            // deliberately not the reqwest client used elsewhere in the server (different
            // major version of the reqwest crate, bundled by `openidconnect` itself).
            // `redirect::Policy::none()` matters for the token endpoint request: following
            // a redirect there would silently drop the POST body/auth.
            http_client: openidconnect::reqwest::ClientBuilder::new()
                .redirect(openidconnect::reqwest::redirect::Policy::none())
                .build()
                .expect("failed to build the OIDC HTTP client"),
            core_client: OnceCell::new(),
            logout_endpoint: OnceCell::new(),
        }
    }

    /// Discovers the IdP's metadata (`.well-known/openid-configuration`) on first use and
    /// caches it for the process's lifetime. Deliberately not called at server startup —
    /// an unreachable IdP must not block `api_server` from booting.
    async fn client(&self) -> Result<&DiscoveredCoreClient, ApiError> {
        self.core_client
            .get_or_try_init(|| async {
                let issuer = IssuerUrl::new(self.config.issuer_url.clone()).map_err(|e| {
                    ApiError::InternalServerError(format!("Invalid OIDC_ISSUER_URL: {e}"))
                })?;
                let metadata = CoreProviderMetadata::discover_async(issuer, &self.http_client)
                    .await
                    .map_err(|e| {
                        ApiError::ServiceUnavailable(format!("OIDC discovery failed: {e}"))
                    })?;
                let redirect_uri = RedirectUrl::new(self.config.redirect_uri()).map_err(|e| {
                    ApiError::InternalServerError(format!("Invalid OIDC_REDIRECT_BASE_URL: {e}"))
                })?;
                Ok(WoodstockClient::from_provider_metadata(
                    metadata,
                    ClientId::new(self.config.client_id.clone()),
                    self.config.client_secret.clone().map(ClientSecret::new),
                )
                .set_redirect_uri(redirect_uri))
            })
            .await
    }

    /// Discovers the IdP's `end_session_endpoint` (RP-Initiated Logout), if it advertises
    /// one, caching the result (including the "doesn't have one" case) for the process's
    /// lifetime. This is a second discovery document fetch, separate from [`Self::client`]:
    /// the `openidconnect` crate models RP-Initiated Logout support as different provider
    /// metadata (`ProviderMetadataWithLogout`), not an addition to `CoreProviderMetadata`.
    async fn logout_endpoint(&self) -> Result<Option<&EndSessionUrl>, ApiError> {
        let endpoint = self
            .logout_endpoint
            .get_or_try_init(|| async {
                let issuer = IssuerUrl::new(self.config.issuer_url.clone()).map_err(|e| {
                    ApiError::InternalServerError(format!("Invalid OIDC_ISSUER_URL: {e}"))
                })?;
                let metadata =
                    ProviderMetadataWithLogout::discover_async(issuer, &self.http_client)
                        .await
                        .map_err(|e| {
                            ApiError::ServiceUnavailable(format!("OIDC discovery failed: {e}"))
                        })?;
                Ok::<_, ApiError>(metadata.additional_metadata().end_session_endpoint.clone())
            })
            .await?;
        Ok(endpoint.as_ref())
    }

    /// Builds the URL to send the browser to so the IdP ends its own SSO session too, not
    /// just the local Woodstock Backup one — without this, the next `/auth/login` silently
    /// re-authenticates the same person via the IdP's still-live session instead of asking
    /// them to sign in again. Returns `None` if the IdP doesn't support RP-Initiated Logout,
    /// in which case the caller should fall back to a purely local logout.
    pub async fn end_session_url(
        &self,
        id_token: &str,
        post_logout_redirect_uri: &str,
    ) -> Result<Option<String>, ApiError> {
        let Some(endpoint) = self.logout_endpoint().await? else {
            return Ok(None);
        };

        let mut request = LogoutRequest::from(endpoint.clone())
            .set_client_id(ClientId::new(self.config.client_id.clone()));

        if let Ok(id_token) = CoreIdToken::from_str(id_token) {
            request = request.set_id_token_hint(&id_token);
        }

        let redirect_uri = PostLogoutRedirectUrl::new(post_logout_redirect_uri.to_string())
            .map_err(|e| {
                ApiError::InternalServerError(format!("Invalid post-logout redirect URL: {e}"))
            })?;
        request = request.set_post_logout_redirect_uri(redirect_uri);

        Ok(Some(request.http_get_url().to_string()))
    }

    /// Builds the URL to redirect the browser to, plus the PKCE/CSRF/nonce secrets the
    /// caller must persist (in the Redis flow store) until the callback arrives.
    pub async fn authorization_request(&self) -> Result<AuthorizationRequest, ApiError> {
        let client = self.client().await?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf_token, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok(AuthorizationRequest {
            authorize_url: authorize_url.to_string(),
            csrf_token: csrf_token.secret().clone(),
            nonce: nonce.secret().clone(),
            pkce_verifier: pkce_verifier.secret().clone(),
        })
    }

    /// Exchanges the authorization code for tokens, then validates the ID token
    /// (signature via the IdP's JWKS, plus `iss`/`aud`/`exp`/`nonce`) and extracts the
    /// admin/identity claims configured in [`OidcConfig`]. Returns an error if the
    /// exchange fails or the ID token does not validate — never returns claims that
    /// haven't passed `openidconnect`'s own verification.
    pub async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: String,
        nonce: String,
    ) -> Result<AuthenticatedIdentity, ApiError> {
        let client = self.client().await?;

        let token_response = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|e| ApiError::Unauthorized(format!("Invalid authorization code: {e}")))?
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
            .request_async(&self.http_client)
            .await
            .map_err(|e| ApiError::Unauthorized(format!("OIDC token exchange failed: {e}")))?;

        let id_token = token_response.id_token().ok_or_else(|| {
            ApiError::Unauthorized("The identity provider did not return an ID token".to_string())
        })?;
        let raw_id_token = id_token.to_string();

        let verifier = client.id_token_verifier();
        let claims = id_token
            .claims(&verifier, &Nonce::new(nonce))
            .map_err(|e| ApiError::Unauthorized(format!("Invalid ID token: {e}")))?;

        let subject = claims.subject().as_str().to_string();

        // Round-trip through JSON so `resolve_claim_path` can walk both a nested claim
        // (Keycloak's `realm_access.roles`) and a flat one (a plain `groups` array)
        // without the caller needing typed access to every possible IdP-specific claim.
        // This only sees `realm_access` etc. at all because `claims` is typed with
        // `ExtraClaims` (see its doc comment) instead of `EmptyAdditionalClaims` — with the
        // latter, those claims are dropped before this line ever runs, no matter what's
        // done with the result.
        let claims_json = serde_json::to_value(claims).map_err(|e| {
            ApiError::InternalServerError(format!("Failed to serialize ID token claims: {e}"))
        })?;

        let is_admin = claim_grants_admin(
            resolve_claim_path(&claims_json, &self.config.admin_role_claim_path),
            &self.config.admin_role_values,
        );

        let identity = resolve_claim_path(&claims_json, &self.config.identity_claim)
            .and_then(|v| v.as_str())
            .map(str::to_lowercase)
            .ok_or_else(|| {
                ApiError::Unauthorized(format!(
                    "ID token is missing the configured identity claim '{}'",
                    self.config.identity_claim
                ))
            })?;

        Ok(AuthenticatedIdentity {
            subject,
            identity,
            is_admin,
            id_token: raw_id_token,
        })
    }
}

#[cfg(test)]
mod tests {
    use openidconnect::core::CoreGenderClaim;
    use openidconnect::IdTokenClaims;

    use super::ExtraClaims;
    use crate::auth::authz::{claim_grants_admin, resolve_claim_path};

    /// Regression test for a real bug: using `openidconnect::core::CoreIdTokenClaims`
    /// (fixed to `EmptyAdditionalClaims`) silently drops any claim outside the OIDC Core
    /// spec at deserialization time — including Keycloak's `realm_access`, which is
    /// exactly the claim `OIDC_ADMIN_ROLE_CLAIM_PATH` reads by default. This reproduces a
    /// real Keycloak ID token payload (captured from a live test instance, `sub`/`sid`
    /// values are that instance's test data, not secrets) and checks that
    /// `realm_access.roles` — and a standard claim, `preferred_username` — both survive
    /// the round trip through `IdTokenClaims<ExtraClaims, _>` and back to JSON.
    #[test]
    fn realm_access_claim_survives_deserialization_with_extra_claims() {
        let payload = serde_json::json!({
            "iss": "http://localhost:8081/realms/woodstock",
            "aud": "woodstock-backup",
            "sub": "cd5fece4-b0a3-4f44-9049-116a26f7262e",
            "exp": 1_788_685_433,
            "iat": 1_788_685_133,
            "realm_access": { "roles": ["offline_access", "admin", "default-roles-woodstock"] },
            "preferred_username": "admin",
            "email": "admin@example.com",
        });

        let claims: IdTokenClaims<ExtraClaims, CoreGenderClaim> =
            serde_json::from_value(payload).expect("should deserialize a real ID token payload");
        let claims_json = serde_json::to_value(&claims).expect("should serialize back to JSON");

        let roles = resolve_claim_path(&claims_json, "realm_access.roles");
        assert!(claim_grants_admin(roles, &["admin".to_string()]));

        assert_eq!(
            resolve_claim_path(&claims_json, "preferred_username").and_then(|v| v.as_str()),
            Some("admin")
        );
    }

    /// Same payload, but deserialized as `openidconnect::core::CoreIdTokenClaims`
    /// (`EmptyAdditionalClaims`) — documents the exact failure this module works around:
    /// `realm_access` is silently gone, while a standard claim (`preferred_username`)
    /// survives fine. If this test ever starts failing (i.e. `realm_access` shows up),
    /// the crate's behavior changed and the `ExtraClaims` workaround may no longer be
    /// needed — but it isn't a reason to remove it without checking first.
    #[test]
    fn same_payload_loses_realm_access_with_empty_additional_claims() {
        let payload = serde_json::json!({
            "iss": "http://localhost:8081/realms/woodstock",
            "aud": "woodstock-backup",
            "sub": "cd5fece4-b0a3-4f44-9049-116a26f7262e",
            "exp": 1_788_685_433,
            "iat": 1_788_685_133,
            "realm_access": { "roles": ["admin"] },
            "preferred_username": "admin",
        });

        let claims: openidconnect::core::CoreIdTokenClaims =
            serde_json::from_value(payload).expect("should deserialize");
        let claims_json = serde_json::to_value(&claims).expect("should serialize back to JSON");

        assert!(resolve_claim_path(&claims_json, "realm_access.roles").is_none());
        assert_eq!(
            resolve_claim_path(&claims_json, "preferred_username").and_then(|v| v.as_str()),
            Some("admin")
        );
    }
}
