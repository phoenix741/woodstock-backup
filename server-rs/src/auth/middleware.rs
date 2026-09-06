//! Resolves the caller's identity for every request behind the authentication layer.
//!
//! [`session_middleware`] runs in front of the REST API and the GraphQL HTTP endpoint
//! (see `api::routes::create_router`) — never in front of `/auth/*`, `/metrics`, or the
//! static SPA fallback. It reads the session cookie, loads the session from Redis,
//! resolves `owned_hosts` fresh from [`woodstock::config::Hosts`], and stores the
//! resulting [`CurrentUser`] in the request's extensions for [`CurrentUser`]'s own
//! [`axum::extract::FromRequestParts`] impl (below) to pick up in handlers, and for the
//! GraphQL/WebSocket wiring in `graphql::schema` to inject into the per-request `Context`.
//!
//! When authentication is disabled (`OidcConfig::enabled == false`), every request is
//! treated as [`CurrentUser::admin_unrestricted`] — identical to the server's behavior
//! before this module existed.

use axum::{
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use tracing::warn;

use crate::api::ApiServerState;

use super::authz::CurrentUser;
use super::session::{load_session, SessionData};

async fn resolve_current_user(state: &ApiServerState, sid: &str) -> Option<CurrentUser> {
    let SessionData {
        subject,
        identity,
        is_admin,
        id_token: _,
    } = load_session(&state.redis_client, sid).await?;

    if is_admin {
        return Some(CurrentUser::admin(subject, identity));
    }

    let owned_hosts = match state.hosts.list_hosts_owned_by(&identity).await {
        Ok(hosts) => hosts,
        Err(e) => {
            warn!("Failed to resolve owned hosts for {identity}: {e}");
            Default::default()
        }
    };
    Some(CurrentUser::restricted_user(subject, identity, owned_hosts))
}

/// How often an open GraphQL WebSocket subscription re-checks that its backing session
/// still exists. A subscription is established once at upgrade time and, unlike REST/HTTP
/// GraphQL requests, never goes through [`session_middleware`] again for the life of the
/// connection — without this poll, logging out or a session expiring would never actually
/// stop an already-open subscription from streaming.
pub const SESSION_REVOCATION_POLL_INTERVAL: std::time::Duration =
    std::time::Duration::from_secs(30);

/// Resolves once the session identified by `sid` has disappeared (logout, TTL expiry, or
/// deletion) — intended to be raced (`tokio::select!`) against a long-lived WebSocket's
/// serve future in `graphql::schema::graphql_ws_handler`, so the connection is dropped
/// shortly after its session stops being valid instead of staying open indefinitely.
pub async fn wait_for_session_revocation(redis_client: redis::Client, sid: String) {
    loop {
        tokio::time::sleep(SESSION_REVOCATION_POLL_INTERVAL).await;
        if load_session(&redis_client, &sid).await.is_none() {
            return;
        }
    }
}

/// Populates `req`'s extensions with a [`CurrentUser`], rejecting with `401` when
/// authentication is enabled and no valid session is present.
pub async fn session_middleware(
    State(state): State<ApiServerState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let current_user = if !state.oidc.enabled {
        CurrentUser::admin_unrestricted()
    } else {
        let sid = jar
            .get(&state.oidc.cookie_name)
            .map(|c| c.value().to_string())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        resolve_current_user(&state, &sid)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?
    };

    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED);
        async move { result }
    }
}
