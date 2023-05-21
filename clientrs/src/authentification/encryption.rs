use base64::{engine::general_purpose, Engine as _};
use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use log::{debug, warn};
use rsa::{
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, error::Error, path::Path};

/// The goal of this module is to provide a way to create and verify a JWT token
/// using the RS256 algorithm.
///
/// The method `generate_rsa_key` is used to create a `public_key` and a `private_key`
/// The methode `generate_password` is used to create a random string of 48 bytes encoded in base64
/// The methode `create_authentification_token is used to create a JWT token using the private_key
/// The methode ``verify_authentification_token` is used to verify a JWT token using the `public_key`
///

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    exp: u64,
    aud: String,
    sub: String,
    hash: String,
}

/// Generate a new RSA key pair and return them as PEM strings
/// The public key is stored in `public_key.pem` and the private key in `private_key.pem`
/// If the keys already exists, nothing is done
///
/// # Arguments
///
/// * `certificate_path` - The path where the keys are stored
///
/// # Return
///
/// * `Result<(), Box<dyn Error>>` - Return an error if the keys can't be generated
///
/// # Errors
///
/// * `std::io::Error` - If the keys can't be generated
///
pub fn generate_rsa_key(certificate_path: &Path) -> Result<(), Box<dyn Error>> {
    debug!("Generate a new RSA key pair");

    let public_key_path = certificate_path.join("public_key.pem");
    let private_key_path = certificate_path.join("private_key.pem");

    // If public of private file not exists, create them
    if public_key_path.exists() && private_key_path.exists() {
        debug!("RSA key pair already exists");
        return Ok(());
    }

    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits)?;
    let public_key = RsaPublicKey::from(&private_key);

    private_key.write_pkcs8_pem_file(private_key_path, LineEnding::LF)?;
    public_key.write_public_key_pem_file(public_key_path, LineEnding::LF)?;

    debug!("RSA key pair generated");

    Ok(())
}

/// Generate a random password of 48 bytes encoded in base64
/// The password is used to create a hash that is stored in the JWT token
///
/// # Arguments
///
/// * data - The data to hash
///
/// # Return
///
/// * `String` - The random password encoded in base64
///
fn create_sha256_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    general_purpose::STANDARD.encode(result)
}

/// Create a new authentification token for a host
/// The token is signed using the private key stored in `private_key.pem`
/// The token contains the hash of the password
///
/// # Arguments
///
/// * `certificate_path` - The path where the keys are stored
/// * `host` - The host for which the token is created
/// * `password` - The password to hash
///
/// # Return
///
/// * `Result<String, Box<dyn Error>>` - Return the JWT token
///
/// # Errors
///
/// * `std::io::Error` - If the keys can't be read
/// * `jsonwebtoken::errors::Error` - If the token can't be created
/// * `std::string::FromUtf8Error` - If the password can't be encoded in UTF-8
/// * `std::string::FromUtf8Error` - If the password can't be encoded in base64
///
pub fn create_authentification_token(
    certificate_path: &Path,
    host: &str,
    password: &str,
) -> Result<String, Box<dyn Error>> {
    debug!("Create a new authentification token for host {host}");

    let private_key_path = certificate_path.join("private_key.pem");
    let private_key = std::fs::read(private_key_path)?;
    let encoding_key = EncodingKey::from_rsa_pem(&private_key)?;

    // Create a sha256 from password
    let password_hash = create_sha256_hash(password);

    let header = Header::new(Algorithm::RS256);
    let payload = Claims {
        iss: "woodstock.shadoware.org".to_string(),
        sub: host.to_string(),
        aud: host.to_string(),
        exp: get_current_timestamp() + 60, // 1 minute
        hash: password_hash,
    };

    let token = encode(&header, &payload, &encoding_key)?;

    debug!("Authentification token for host {host} created");

    Ok(token)
}

/// Verify the authentification token for a host
/// The token is verified using the public key stored in `public_key.pem`
/// The token is valid if the hash of the password is the same as the one stored in the token
/// The token is also valid if the expiration date is not reached
///
/// # Arguments
///
/// * `certificate_path` - The path where the keys are stored
/// * `host` - The host for which the token is created
/// * `token` - The JWT token to verify
/// * `password` - The password to hash
///
/// # Return
///
/// * `Result<(), Box<dyn Error>>` - Return an error if the token is invalid
///
/// # Errors
///
/// * `std::io::Error` - If the keys can't be read
/// * `jsonwebtoken::errors::Error` - If the token can't be decoded
/// * `std::string::FromUtf8Error` - If the password can't be encoded in UTF-8
/// * `std::string::FromUtf8Error` - If the password can't be encoded in base64
/// * `std::io::Error` - If the password is invalid
///
pub fn verify_authentification_token(
    certificate_path: &Path,
    host: &str,
    token: &str,
    password: &str,
) -> Result<(), Box<dyn Error>> {
    debug!("Verify the authentification token for host {host}");

    let public_key_path = certificate_path.join("public_key.pem");
    let public_key = std::fs::read(public_key_path)?;
    let decoding_key = DecodingKey::from_rsa_pem(&public_key)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.iss = Some(HashSet::from(["woodstock.shadoware.org".to_string()]));
    validation.sub = Some(host.to_string());
    validation.aud = Some(HashSet::from([host.to_string()]));
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    let password_hash = create_sha256_hash(password);

    if token_data.claims.hash != password_hash {
        warn!("Invalid password for host {host}");
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid password",
        )));
    }

    debug!("Authentification token for host {host} is valid");

    Ok(())
}
