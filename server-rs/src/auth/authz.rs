//! Authorization: who is making the request, and what are they allowed to touch.
//!
//! [`CurrentUser`] is resolved once per request (HTTP) or once per connection/subscription
//! (WebSocket) by [`super::middleware::session_middleware`] and the GraphQL context wiring
//! in `graphql::schema`, then consulted by every REST handler and GraphQL resolver that
//! needs to know whether the caller may see or act on a given host. Nothing here validates
//! tokens or signatures — that is delegated entirely to the `openidconnect` crate in
//! `super::oidc`; this module only turns already-validated claims into an access decision.

use std::collections::HashSet;

use serde_json::Value;

use crate::api::ApiError;

/// The authenticated (or, when OIDC is disabled, implicit) caller of the current request.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// OIDC `sub` claim. Empty when authentication is disabled.
    pub subject: String,
    /// Value of the configured identity claim (e.g. `preferred_username`), lowercased.
    /// Empty when authentication is disabled.
    pub identity: String,
    pub is_admin: bool,
    /// `None` means unrestricted access (admin, or authentication disabled).
    /// `Some(set)` is the exact set of hostnames this user owns.
    pub owned_hosts: Option<HashSet<String>>,
}

impl CurrentUser {
    /// The implicit user used when `OIDC_ENABLED=false` — unrestricted access, identical
    /// to the server's behavior before authentication existed.
    #[must_use]
    pub fn admin_unrestricted() -> Self {
        Self {
            subject: String::new(),
            identity: String::new(),
            is_admin: true,
            owned_hosts: None,
        }
    }

    /// Builds an authenticated non-admin user restricted to `owned_hosts`.
    #[must_use]
    pub fn restricted_user(
        subject: String,
        identity: String,
        owned_hosts: HashSet<String>,
    ) -> Self {
        Self {
            subject,
            identity,
            is_admin: false,
            owned_hosts: Some(owned_hosts),
        }
    }

    /// Builds an authenticated admin user.
    #[must_use]
    pub fn admin(subject: String, identity: String) -> Self {
        Self {
            subject,
            identity,
            is_admin: true,
            owned_hosts: None,
        }
    }

    /// Whether this user may see/act on `hostname`.
    #[must_use]
    pub fn can_see_host(&self, hostname: &str) -> bool {
        self.is_admin
            || self
                .owned_hosts
                .as_ref()
                .is_some_and(|hosts| hosts.contains(hostname))
    }

    /// Whether this user may see/act on at least one of `hostnames` (used for
    /// operations that fan out across several hosts, e.g. an archive profile run).
    #[must_use]
    pub fn can_see_any_host(&self, hostnames: &[String]) -> bool {
        self.is_admin || hostnames.iter().any(|h| self.can_see_host(h))
    }

    /// Rejects with [`ApiError::Forbidden`] unless the user is an admin.
    pub fn require_admin(&self) -> Result<(), ApiError> {
        if self.is_admin {
            Ok(())
        } else {
            Err(ApiError::Forbidden(
                "This operation requires administrator privileges".to_string(),
            ))
        }
    }

    /// Rejects with [`ApiError::NotFound`] unless the user may see `hostname`. `NotFound`
    /// rather than `Forbidden` is deliberate on read paths: it avoids confirming to a
    /// non-owner that a given hostname exists at all.
    pub fn require_can_see_host(&self, hostname: &str) -> Result<(), ApiError> {
        if self.can_see_host(hostname) {
            Ok(())
        } else {
            Err(ApiError::NotFound(format!(
                "Can't find the host with the name {hostname}"
            )))
        }
    }
}

/// Resolves a dotted claim path (e.g. `realm_access.roles`, or a flat `groups`) against a
/// decoded ID token claims object. Handles both a Keycloak-style nested object
/// (`realm_access: { roles: [...] }`) and a flat top-level array/string claim identically.
pub fn resolve_claim_path<'a>(claims: &'a Value, dotted_path: &str) -> Option<&'a Value> {
    let mut current = claims;
    for segment in dotted_path.split('.') {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}

/// Whether the claim value resolved by `admin_role_claim_path` (an array of strings, or a
/// single string) intersects any of `admin_role_values`.
#[must_use]
pub fn claim_grants_admin(claim_value: Option<&Value>, admin_role_values: &[String]) -> bool {
    let Some(value) = claim_value else {
        return false;
    };
    match value {
        Value::Array(values) => values.iter().any(|v| {
            v.as_str()
                .is_some_and(|s| admin_role_values.iter().any(|a| a == s))
        }),
        Value::String(s) => admin_role_values.iter().any(|a| a == s),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolves_nested_claim_path() {
        let claims = json!({ "realm_access": { "roles": ["user", "admin"] } });
        let resolved = resolve_claim_path(&claims, "realm_access.roles").unwrap();
        assert!(claim_grants_admin(Some(resolved), &["admin".to_string()]));
    }

    #[test]
    fn resolves_flat_claim_path() {
        let claims = json!({ "groups": ["/woodstock-admins"] });
        let resolved = resolve_claim_path(&claims, "groups").unwrap();
        assert!(!claim_grants_admin(Some(resolved), &["admin".to_string()]));
        assert!(claim_grants_admin(
            Some(resolved),
            &["/woodstock-admins".to_string()]
        ));
    }

    #[test]
    fn missing_claim_path_is_not_admin() {
        let claims = json!({ "sub": "abc" });
        assert!(resolve_claim_path(&claims, "realm_access.roles").is_none());
        assert!(!claim_grants_admin(None, &["admin".to_string()]));
    }

    #[test]
    fn can_see_host_truth_table() {
        let admin = CurrentUser::admin("s".into(), "i".into());
        assert!(admin.can_see_host("anything"));
        assert!(admin.require_admin().is_ok());

        let mut owned = HashSet::new();
        owned.insert("myhost".to_string());
        let user = CurrentUser::restricted_user("s".into(), "i".into(), owned);
        assert!(user.can_see_host("myhost"));
        assert!(!user.can_see_host("other"));
        assert!(user.require_admin().is_err());
        assert!(user.require_can_see_host("myhost").is_ok());
        assert!(user.require_can_see_host("other").is_err());
    }

    #[test]
    fn unrestricted_admin_used_when_auth_disabled() {
        let user = CurrentUser::admin_unrestricted();
        assert!(user.is_admin);
        assert!(user.owned_hosts.is_none());
        assert!(user.can_see_host("anything"));
    }
}
