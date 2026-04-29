use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::config::ClientConfig;
use eyre::{eyre, Result};
use woodstock::utils::encryption;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Represents the JWT claims structure used for authentication.
struct Claims {
    /// Issuer of the token, typically the hostname
    iss: String,
    /// Audience of the token, typically the hostname
    aud: String,
    /// Expiration timestamp
    exp: u64,
    /// Subject of the token, typically a UUID
    sub: String,

    /// Session identifier for the authenticated session
    session_id: String,
    /// Flag indicating whether the session is authenticated
    is_authenticated: bool,
}

struct ContextData {
    /// Expiration timestamp for the session context
    exp: u64,
}

/// The goal of this module is to provide a way to create and verify a JWT token
/// using the HS256 algorithm.
pub struct Service {
    /// Map of active session contexts, indexed by session ID
    context: HashMap<String, Arc<Mutex<ContextData>>>,
    /// Path to the certificate file used for authentication
    certificate_path: PathBuf,
    /// Client hostname
    hostname: String,
    /// Client password for authentication
    password: String,
    /// Key used for JWT token encoding
    encoding_secret: EncodingKey,
    /// Key used for JWT token decoding
    decoding_secret: DecodingKey,
    /// Timeout for backup operations in seconds
    backup_timeout: u64,
    /// Maximum duration for which a backup session can remain active, in seconds
    max_backup_seconds: u64,
    /// Flag indicating whether restoration operations are disabled
    disable_restauration: bool,
}

impl Service {
    /// Creates a new instance of the `Service` struct.
    ///
    /// # Arguments
    ///
    /// * `certificate_path` - The path to the certificate file.
    /// * `config` - The client configuration.
    ///
    /// # Returns
    ///
    /// A new instance of the `Service` struct.
    #[must_use]
    pub fn new(certificate_path: &Path, config: &ClientConfig) -> Self {
        Self {
            context: HashMap::new(),
            certificate_path: certificate_path.to_path_buf(),
            hostname: config.hostname.to_string(),
            password: config.password.to_string(),
            encoding_secret: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_secret: DecodingKey::from_secret(config.secret.as_bytes()),
            backup_timeout: config.backup_timeout,
            max_backup_seconds: config.max_backup_seconds,
            disable_restauration: config.disable_restauration,
        }
    }

    /// Authenticates the server using the provided token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token to authenticate.
    ///
    /// # Returns
    ///
    /// The authenticated token as a `String`, or an error if authentication fails.
    ///
    /// # Errors
    ///
    /// An error is returned if the token is invalid.
    ///
    pub fn authenticate(&mut self, token: &str) -> Result<String> {
        debug!("Try to authenticate the server for host {}", self.hostname);

        // Check the token validity
        encryption::verify_authentification_token(
            self.certificate_path.as_path(),
            &self.hostname,
            token,
            &self.password,
        )?;

        let uuid = Uuid::new_v4();
        let uuid = uuid.to_string();

        let header = Header::new(Algorithm::HS256);
        let payload = Claims {
            iss: self.hostname.to_string(),
            aud: self.hostname.to_string(),
            sub: uuid.clone(),
            exp: get_current_timestamp() + self.max_backup_seconds,

            session_id: uuid.clone(),
            is_authenticated: true,
        };

        let token = encode(&header, &payload, &self.encoding_secret)?;

        info!(
            "Authentification of the host {} successfull ({uuid})",
            self.hostname
        );

        self.context.insert(
            uuid,
            Arc::new(Mutex::new(ContextData {
                exp: get_current_timestamp() + self.backup_timeout,
            })),
        );

        Ok(token)
    }

    /// Checks the context of the provided token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token to check.
    ///
    /// # Returns
    ///
    /// The session ID as a `String`, or an error if the context is invalid.
    ///
    /// # Errors
    ///
    /// An error is returned if the token is invalid.
    ///
    pub async fn check_context(&self, token: &str) -> Result<String> {
        debug!("Check the context of the token");

        // Decode JWT Token
        let mut validation = Validation::new(Algorithm::HS256);
        validation.iss = Some(HashSet::from([self.hostname.to_string()]));
        validation.aud = Some(HashSet::from([self.hostname.to_string()]));
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &self.decoding_secret, &validation)?;

        let context = self.context.get(token_data.claims.session_id.as_str());
        if let Some(context) = context {
            let mut context = context.lock().await;

            if context.exp < get_current_timestamp() {
                warn!(
                    "Session id of the token {} expired",
                    token_data.claims.session_id
                );

                return Err(eyre!("Session expired"));
            }
            context.exp = get_current_timestamp() + self.backup_timeout;
        } else {
            warn!(
                "Session id of the token {} invalid",
                token_data.claims.session_id
            );

            return Err(eyre!("Session not found"));
        }

        if !token_data.claims.is_authenticated {
            warn!("Claim is_authenticated is not activated in the token");

            return Err(eyre!("Session not authenticated"));
        }

        debug!("The session id {} is valid", token_data.claims.session_id);

        Ok(token_data.claims.session_id)
    }

    #[must_use]
    /// Checks if restoration operations are disabled in the current configuration.
    ///
    /// # Returns
    ///
    /// Returns `true` if restoration operations are disabled, `false` otherwise.
    pub fn is_restauration_disabled(&self) -> bool {
        self.disable_restauration
    }

    #[must_use]
    /// Validates if a session with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to validate.
    ///
    /// # Returns
    ///
    /// Returns `true` if the session exists, `false` otherwise.
    pub fn validate_session(&self, session_id: &str) -> bool {
        self.context.contains_key(session_id)
    }

    /// Logs out the session associated with the provided token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token to log out.
    ///
    /// # Returns
    ///
    /// An empty result if the logout is successful, or an error if the token is invalid.
    ///
    /// # Errors
    ///
    /// An error is returned if the token is invalid.
    ///
    pub fn logout(&mut self, session_id: &str) -> Result<()> {
        debug!("Logout the session associated with the token {session_id}");

        self.context.remove(session_id);

        info!("Session {} logged out", session_id);
        Ok(())
    }
}
