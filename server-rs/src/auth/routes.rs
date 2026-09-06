//! Public (unauthenticated) routes for the OIDC login/logout flow, plus two small
//! endpoints the SPA uses to decide what to show before/without a session.

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::{Deserialize, Serialize};
use time::Duration;
use tracing::warn;

use crate::api::{ApiError, ApiServerState};

use super::authz::CurrentUser;
use super::session::{consume_flow, create_flow, create_session, delete_session, load_session};

pub fn router() -> Router<ApiServerState> {
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", post(logout))
        .route("/api/auth/config", get(auth_config))
        .route("/api/auth/me", get(me))
}

#[derive(Deserialize)]
struct LoginQuery {
    return_to: Option<String>,
}

/// Only a same-origin, relative path may be used as the post-login redirect target.
/// `return_to` is attacker-controlled (it's a query parameter on `/auth/login`, echoed
/// back into a `Location:` header right after a real login) — without this, it's an open
/// redirect. Rejects anything that isn't a plain relative path: no scheme (`javascript:`,
/// `https://evil.example/`), no protocol-relative `//evil.example` (a browser treats a
/// leading `//` as an absolute URL, not a path), and no backslash (some browsers normalize
/// `/\evil.example` to `//evil.example`).
fn sanitize_return_to(return_to: Option<String>) -> String {
    match return_to {
        Some(path) if path.starts_with('/') && !path.starts_with("//") && !path.contains('\\') => {
            path
        }
        _ => "/".to_string(),
    }
}

fn session_cookie(
    name: &str,
    value: String,
    max_age: Option<Duration>,
    secure: bool,
) -> Cookie<'static> {
    let mut cookie = Cookie::new(name.to_string(), value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(secure);
    cookie.set_same_site(SameSite::Lax);
    if let Some(max_age) = max_age {
        cookie.set_max_age(max_age);
    }
    cookie
}

async fn login(
    State(state): State<ApiServerState>,
    Query(query): Query<LoginQuery>,
) -> Result<Response, ApiError> {
    if !state.oidc.enabled {
        return Err(ApiError::BadRequest(
            "Authentication is not enabled on this server".to_string(),
        ));
    }

    let request = state.oidc_client.authorization_request().await?;

    create_flow(
        &state.redis_client,
        &request.csrf_token,
        request.pkce_verifier,
        request.nonce,
        sanitize_return_to(query.return_to),
    )
    .await?;

    Ok(Redirect::to(&request.authorize_url).into_response())
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

async fn callback(
    State(state): State<ApiServerState>,
    Query(query): Query<CallbackQuery>,
    jar: CookieJar,
) -> Result<Response, ApiError> {
    if let Some(error) = query.error {
        warn!(
            "OIDC callback returned an error: {error} ({})",
            query.error_description.unwrap_or_default()
        );
        return Err(ApiError::Unauthorized(format!(
            "Authentication failed: {error}"
        )));
    }

    let code = query
        .code
        .ok_or_else(|| ApiError::BadRequest("Missing authorization code".to_string()))?;
    let csrf_state = query
        .state
        .ok_or_else(|| ApiError::BadRequest("Missing state parameter".to_string()))?;

    let (pkce_verifier, nonce, return_to) = consume_flow(&state.redis_client, &csrf_state)
        .await
        .ok_or_else(|| {
            ApiError::Unauthorized(
                "Login attempt expired or was already used; please try again".to_string(),
            )
        })?;

    let identity = state
        .oidc_client
        .exchange_code(code, pkce_verifier, nonce)
        .await?;

    let sid = create_session(
        &state.redis_client,
        &super::session::SessionData {
            subject: identity.subject,
            identity: identity.identity,
            is_admin: identity.is_admin,
            id_token: identity.id_token,
        },
        state.oidc.session_ttl_secs,
    )
    .await?;

    let cookie = session_cookie(
        &state.oidc.cookie_name,
        sid,
        Some(Duration::seconds(state.oidc.session_ttl_secs as i64)),
        state.oidc.cookie_secure,
    );

    Ok((jar.add(cookie), Redirect::to(&return_to)).into_response())
}

#[derive(Serialize)]
struct LogoutResponse {
    /// Where the SPA should navigate to next. Either the IdP's end-session URL (so its own
    /// SSO session ends too — otherwise the next `/auth/login` would silently re-authenticate
    /// the same person without a login prompt) or, if that isn't available, `/`.
    redirect_to: String,
}

async fn logout(State(state): State<ApiServerState>, jar: CookieJar) -> Response {
    let mut redirect_to = "/".to_string();

    if let Some(cookie) = jar.get(&state.oidc.cookie_name) {
        let sid = cookie.value().to_string();

        if state.oidc.enabled {
            if let Some(session) = load_session(&state.redis_client, &sid).await {
                match state
                    .oidc_client
                    .end_session_url(&session.id_token, &state.oidc.post_logout_redirect_uri())
                    .await
                {
                    Ok(Some(url)) => redirect_to = url,
                    Ok(None) => {}
                    Err(e) => warn!("Failed to build the IdP end-session URL: {e}"),
                }
            }
        }

        delete_session(&state.redis_client, &sid).await;
    }

    let expired = session_cookie(
        &state.oidc.cookie_name,
        String::new(),
        Some(Duration::ZERO),
        state.oidc.cookie_secure,
    );
    (jar.remove(expired), Json(LogoutResponse { redirect_to })).into_response()
}

#[derive(Serialize)]
struct AuthConfigResponse {
    enabled: bool,
}

async fn auth_config(State(state): State<ApiServerState>) -> Json<AuthConfigResponse> {
    Json(AuthConfigResponse {
        enabled: state.oidc.enabled,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MeResponse {
    authenticated: bool,
    identity: Option<String>,
    is_admin: bool,
}

async fn me(State(state): State<ApiServerState>, jar: CookieJar) -> Json<MeResponse> {
    if !state.oidc.enabled {
        let user = CurrentUser::admin_unrestricted();
        return Json(MeResponse {
            authenticated: true,
            identity: None,
            is_admin: user.is_admin,
        });
    }

    let Some(cookie) = jar.get(&state.oidc.cookie_name) else {
        return Json(MeResponse {
            authenticated: false,
            identity: None,
            is_admin: false,
        });
    };

    match load_session(&state.redis_client, cookie.value()).await {
        Some(session) => Json(MeResponse {
            authenticated: true,
            identity: Some(session.identity),
            is_admin: session.is_admin,
        }),
        None => Json(MeResponse {
            authenticated: false,
            identity: None,
            is_admin: false,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_return_to;

    #[test]
    fn accepts_a_plain_relative_path() {
        assert_eq!(sanitize_return_to(Some("/devices".to_string())), "/devices");
        assert_eq!(
            sanitize_return_to(Some("/backups/host1?tab=files".to_string())),
            "/backups/host1?tab=files"
        );
    }

    #[test]
    fn rejects_absolute_and_protocol_relative_urls() {
        assert_eq!(
            sanitize_return_to(Some("https://evil.example/".to_string())),
            "/"
        );
        assert_eq!(sanitize_return_to(Some("//evil.example".to_string())), "/");
        assert_eq!(
            sanitize_return_to(Some("javascript:alert(1)".to_string())),
            "/"
        );
        assert_eq!(sanitize_return_to(Some("/\\evil.example".to_string())), "/");
    }

    #[test]
    fn defaults_to_root_when_absent() {
        assert_eq!(sanitize_return_to(None), "/");
    }
}
