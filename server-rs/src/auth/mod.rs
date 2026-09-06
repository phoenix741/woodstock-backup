//! OpenID Connect authentication for the public API server (`api_server`).
//!
//! See `docs/developer_guide/AUTHENTICATION.md` for the full design. Summary: an external
//! IdP (Keycloak, Azure AD, or any OIDC-compliant provider) is the identity source — this
//! module never invents or stores a password. `api_server` performs the Authorization
//! Code + PKCE exchange itself (a BFF pattern) and hands the browser only an opaque
//! HttpOnly session cookie backed by Redis; the IdP's tokens never reach the SPA.
//!
//! - [`authz`]: [`authz::CurrentUser`], the per-request access decision (admin vs. the set
//!   of hosts a non-admin user owns).
//! - [`oidc`]: the OIDC client — discovery, PKCE, code exchange, ID token validation (all
//!   delegated to the `openidconnect` crate, nothing hand-rolled).
//! - [`session`]: Redis-backed session and login-flow storage.
//! - [`middleware`]: resolves [`authz::CurrentUser`] for every request behind the
//!   authentication layer.
//! - [`routes`]: the public `/auth/*` and `/api/auth/*` endpoints.

pub mod authz;
pub mod middleware;
pub mod oidc;
pub mod routes;
pub mod session;
