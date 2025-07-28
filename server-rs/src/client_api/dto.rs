//! Data Transfer Objects for Client API
//!
//! These DTOs match exactly the TypeScript definitions in nestjs/apps/clientApi/src/hosts/hosts.dto.ts

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct Ipv4Addr {
    #[validate(ip(v4, message = "Must be a valid IPv4 address"))]
    pub addr: String,
    #[validate(length(min = 1, message = "Netmask cannot be empty"))]
    pub netmask: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct Ipv6Addr {
    #[validate(ip(v6, message = "Must be a valid IPv6 address"))]
    pub addr: String,
    #[validate(length(min = 1, message = "Netmask cannot be empty"))]
    pub netmask: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct InterfaceInformation {
    #[validate(length(min = 1, message = "Interface name cannot be empty"))]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub ipv4: Option<Ipv4Addr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub ipv6: Option<Ipv6Addr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct RegisterClient {
    #[validate(length(min = 1, message = "At least one address must be provided"))]
    #[validate(nested)]
    pub addresses: Vec<InterfaceInformation>,
    #[validate(range(min = 1, max = 65535, message = "Port must be between 1 and 65535"))]
    pub port: u16,
    #[validate(length(min = 1, message = "Version cannot be empty"))]
    pub version: String,
}
