//! Business logic services for API consistency
//!
//! These services ensure that REST and GraphQL APIs return identical data
//! by providing a shared business logic layer.

pub mod backups;
pub mod certificate;
pub mod files;
pub mod hosts;
pub mod metrics;
pub mod queue;
pub mod server;

#[cfg(test)]
mod test_certificates;

#[cfg(test)]
mod test_certificate_compatibility;

#[cfg(test)]
mod test_certificate_optimization;

#[cfg(test)]
mod test_certificate_validation;

#[cfg(test)]
mod test_certificate_final_validation;

pub use backups::BackupsService;
pub use certificate::CertificateService;
pub use files::FilesService;
pub use hosts::HostsService;
pub use metrics::MetricsService;
pub use queue::{JobStatus, QueueService, QueueStats};
pub use server::{LogFileStream, ServerInfo, ServerService};
