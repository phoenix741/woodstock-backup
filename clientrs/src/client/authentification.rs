use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::client::config::ClientConfig;
use crate::utils::encryption;
use eyre::{eyre, Result};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    aud: String,
    exp: u64,
    sub: String,

    session_id: String,
    is_authenticated: bool,
}

/// The goal of this module is to provide a way to create and verify a JWT token
/// using the HS256 algorithm.
pub struct Service {
    context: HashSet<String>,
    certificate_path: PathBuf,
    hostname: String,
    password: String,
    encoding_secret: EncodingKey,
    decoding_secret: DecodingKey,
    backup_timeout: u64,
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
            context: HashSet::new(),
            certificate_path: certificate_path.to_path_buf(),
            hostname: config.hostname.to_string(),
            password: config.password.to_string(),
            encoding_secret: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_secret: DecodingKey::from_secret(config.secret.as_bytes()),
            backup_timeout: config.backup_timeout,
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
            exp: get_current_timestamp() + self.backup_timeout,

            session_id: uuid.clone(),
            is_authenticated: true,
        };

        let token = encode(&header, &payload, &self.encoding_secret)?;

        info!(
            "Authentification of the host {} successfull ({uuid})",
            self.hostname
        );

        self.context.insert(uuid);

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
    pub fn check_context(&self, token: &str) -> Result<String> {
        debug!("Check the context of the token");

        // Decode JWT Token
        let mut validation = Validation::new(Algorithm::HS256);
        validation.iss = Some(HashSet::from([self.hostname.to_string()]));
        validation.aud = Some(HashSet::from([self.hostname.to_string()]));
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &self.decoding_secret, &validation)?;

        if !self.context.contains(token_data.claims.session_id.as_str()) {
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
    pub fn validate_session(&self, session_id: &str) -> bool {
        self.context.contains(session_id)
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
