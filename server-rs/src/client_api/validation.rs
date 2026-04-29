//! Request validation utilities for the Client API
//!
//! This module provides validation extractors and error handling that match
//! the NestJS ValidationPipe behavior.

use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use std::future::Future;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

/// Validation error that can be returned from validated extractors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] JsonRejection),
    #[error("Validation failed: {0}")]
    ValidationFailed(#[from] ValidationErrors),
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ValidationError::JsonError(err) => {
                (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", err))
            }
            ValidationError::ValidationFailed(err) => {
                let error_details = err
                    .field_errors()
                    .iter()
                    .map(|(field, errors)| {
                        let messages: Vec<String> = errors
                            .iter()
                            .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
                            .collect();
                        format!("{}: {}", field, messages.join(", "))
                    })
                    .collect::<Vec<String>>()
                    .join("; ");

                (
                    StatusCode::BAD_REQUEST,
                    format!("Validation failed: {}", error_details),
                )
            }
        };

        (status, error_message).into_response()
    }
}

/// A validated JSON extractor that performs request validation
///
/// This extractor automatically validates the request body using the `validator` crate,
/// providing behavior similar to NestJS's ValidationPipe.
///
/// # Example
///
/// ```rust
/// use woodstock_server_rs::client_api::{dto::RegisterClient, validation::ValidatedJson};
///
/// async fn handler(ValidatedJson(payload): ValidatedJson<RegisterClient>) {
///     // payload is guaranteed to be valid
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = ValidationError;

    fn from_request(
        req: Request,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let Json(value) = Json::<T>::from_request(req, state).await?;
            value.validate()?;
            Ok(ValidatedJson(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_api::dto::{InterfaceInformation, Ipv4Addr, RegisterClient};
    use axum::{body::Body, extract::Request, http::Method};
    use serde_json::json;

    #[tokio::test]
    async fn test_validated_json_success() {
        let payload = RegisterClient {
            addresses: vec![InterfaceInformation {
                name: "eth0".to_string(),
                ipv4: Some(Ipv4Addr {
                    addr: "192.168.1.100".to_string(),
                    netmask: "255.255.255.0".to_string(),
                }),
                ipv6: None,
            }],
            port: 8080,
            version: "2.0.0".to_string(),
        };

        let body = serde_json::to_string(&payload).unwrap();
        let req = Request::builder()
            .method(Method::POST)
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedJson::<RegisterClient>::from_request(req, &()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validated_json_validation_failure() {
        let invalid_payload = json!({
            "addresses": [],  // Empty addresses should fail validation
            "port": 0,        // Invalid port
            "version": ""     // Empty version
        });

        let body = serde_json::to_string(&invalid_payload).unwrap();
        let req = Request::builder()
            .method(Method::POST)
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedJson::<RegisterClient>::from_request(req, &()).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            ValidationError::ValidationFailed(_) => {
                // Expected validation failure
            }
            _ => panic!("Expected validation failure, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_validated_json_invalid_json() {
        let req = Request::builder()
            .method(Method::POST)
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();

        let result = ValidatedJson::<RegisterClient>::from_request(req, &()).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            ValidationError::JsonError(_) => {
                // Expected JSON parsing failure
            }
            _ => panic!("Expected JSON error, got: {:?}", error),
        }
    }
}
