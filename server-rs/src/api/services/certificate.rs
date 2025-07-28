//! Certificate service for generating X.509 certificates and CA infrastructure
//!
//! This service implements a complete PKI system where:
//! - Root CA certificates are self-signed
//! - All other certificates are signed by the CA
//! - Proper certificate chains are maintained
//!
//! ## TypeScript Compatibility
//!
//! This implementation is designed to be functionally identical to the TypeScript
//! certificate service found in `nestjs/libs/shared/src/authentification/certificate.service.ts`.
//!
//! ### Certificate Structure (matching TypeScript exactly):
//! - `rootCA.pem/key` - Root CA certificate (self-signed)
//! - `https.pem/key` - HTTPS certificate (signed by root CA)
//! - `{host}_client.pem/key` - Host client certificate (signed by root CA)
//! - `{host}_ca.pem/key` - Host CA certificate (self-signed, NOT signed by root CA)
//! - `{host}_server.pem/key` - Host server certificate (signed by host CA)
//! - `{host}_https.pem/key` - Host HTTPS certificate (signed by root CA)
//!
//! ### Key Features:
//! - Certificate attributes match TypeScript implementation (FR, Paris, Woodstock Backup)
//! - IP address detection using Rust's standard library (more robust than regex)
//! - Subject Alternative Names (SAN) properly configured for IP vs hostname
//! - Extensions configured correctly (keyUsage, extKeyUsage, basicConstraints)
//! - 10-year validity period matching TypeScript
//! - Idempotent generation (won't overwrite existing certificates)
//! - Automatic dependency resolution (generates required CAs automatically)

use eyre::{eyre, Result};
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose, SanType,
};
use rustls::pki_types::{pem::PemObject, CertificateDer};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tokio::fs;
use tracing::info;
use woodstock::config::Configuration;

/// Certificate service for managing PKI infrastructure
#[derive(Clone)]
pub struct CertificateService {
    certificate_path: PathBuf,
}

impl CertificateService {
    /// Create new CertificateService instance
    pub fn new(config: Arc<Configuration>) -> Self {
        // Use the certificates path from woodstock-rs configuration
        let certificate_path = config.path.certificates_path.clone();
        Self { certificate_path }
    }

    /// Create certificate attributes matching the TypeScript implementation
    fn create_certificate_attributes(common_name: &str) -> DistinguishedName {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, "FR");
        dn.push(DnType::StateOrProvinceName, "Paris");
        dn.push(DnType::LocalityName, "Paris");
        dn.push(DnType::OrganizationName, "Woodstock Backup");
        dn.push(DnType::OrganizationalUnitName, "Woodstock Backup");
        dn.push(DnType::CommonName, common_name);
        dn
    }

    /// Check if a string is an IP address (IPv4 or IPv6)
    pub fn is_ip(hostname: &str) -> bool {
        hostname.parse::<IpAddr>().is_ok()
    }

    /// Check if certificate files exist
    async fn cert_files_exist(&self, cert_path: &Path, key_path: &Path) -> bool {
        // Use async file existence checks to avoid blocking the runtime.
        let (cert_exists, key_exists) =
            tokio::join!(fs::try_exists(cert_path), fs::try_exists(key_path));
        cert_exists.unwrap_or(false) && key_exists.unwrap_or(false)
    }

    /// Create subject alternative names based on hostname
    fn create_subject_alt_names(hostname: &str) -> Result<Vec<SanType>> {
        if Self::is_ip(hostname) {
            // SAFETY: is_ip() guarantees the hostname is a valid IP address
            Ok(vec![SanType::IpAddress(hostname.parse().unwrap())])
        } else {
            let dns = SanType::DnsName(
                hostname
                    .try_into()
                    .map_err(|e| eyre!("Invalid DNS name '{hostname}': {e:?}"))?,
            );
            Ok(vec![dns])
        }
    }

    /// Load root CA certificate and create issuer for signing other certificates
    pub async fn load_ca_issuer(&self) -> Result<Issuer<'static, KeyPair>> {
        let cert_path = self.certificate_path.join("rootCA.pem");
        let key_path = self.certificate_path.join("rootCA.key");

        // Ensure CA exists
        if !self.cert_files_exist(&cert_path, &key_path).await {
            return Err(eyre!("Root CA does not exist. Generate it first."));
        }

        // Read the key pair
        let key_pem = fs::read_to_string(&key_path).await?;
        let key_pair = KeyPair::from_pem(&key_pem)?;

        // Read and parse existing certificate to extract params
        let cert_pem = fs::read_to_string(&cert_path).await?;
        let ca_params = self.parse_certificate_params(&cert_pem)?;

        // Create issuer from parsed CA params
        Ok(Issuer::new(ca_params, key_pair))
    }

    /// Create a certificate (self-signed for CA or signed by issuer for others)
    async fn create_certificate(
        &self,
        hostname: &str,
        is_server: bool,
        cert_path: &Path,
        key_path: &Path,
        issuer: Option<&Issuer<'_, KeyPair>>,
    ) -> Result<()> {
        // Create certificate parameters with proper attributes
        let mut params = CertificateParams::new(vec![hostname.to_string()])?;
        params.distinguished_name = Self::create_certificate_attributes(hostname);

        // Set validity period (10 years to match TypeScript)
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(365 * 10);

        // Generate key pair
        let key_pair = KeyPair::generate()?;

        let cert = if let Some(issuer) = issuer {
            // Certificate signed by issuer (CA or host CA)
            params.is_ca = IsCa::NoCa;

            // Add subject alternative names
            params.subject_alt_names = Self::create_subject_alt_names(hostname)?;

            if is_server {
                // Server authentication certificate
                params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
                params.key_usages = vec![
                    KeyUsagePurpose::DigitalSignature,
                    KeyUsagePurpose::KeyEncipherment,
                ];
            } else {
                // Client authentication certificate
                params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
            }

            params.signed_by(&key_pair, issuer)?
        } else {
            // Self-signed CA certificate
            params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
            params.key_usages = vec![KeyUsagePurpose::KeyCertSign];
            params.self_signed(&key_pair)?
        };

        // Write certificate and key
        fs::write(cert_path, cert.pem()).await?;
        fs::write(key_path, key_pair.serialize_pem()).await?;

        Ok(())
    }

    /// Create an HTTPS certificate (specialized method matching TypeScript #createHttpsCertificate)
    async fn create_https_certificate(
        &self,
        hostname: &str,
        cert_path: &Path,
        key_path: &Path,
        issuer: &Issuer<'_, KeyPair>,
    ) -> Result<()> {
        // Create certificate parameters with proper attributes
        let mut params = CertificateParams::new(vec![hostname.to_string()])?;
        params.distinguished_name = Self::create_certificate_attributes(hostname);

        // Set validity period (10 years to match TypeScript)
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(365 * 10);

        // Generate key pair
        let key_pair = KeyPair::generate()?;

        // Configure for HTTPS server certificate (matching TypeScript)
        params.is_ca = IsCa::NoCa;
        params.subject_alt_names = Self::create_subject_alt_names(hostname)?;
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];

        let cert = params.signed_by(&key_pair, issuer)?;

        // Write certificate and key
        fs::write(cert_path, cert.pem()).await?;
        fs::write(key_path, key_pair.serialize_pem()).await?;

        Ok(())
    }

    /// Generate root CA certificate (rootCA.pem, rootCA.key)
    pub async fn generate_certificate(&self) -> Result<()> {
        let cert_path = self.certificate_path.join("rootCA.pem");
        let key_path = self.certificate_path.join("rootCA.key");

        if !self.cert_files_exist(&cert_path, &key_path).await {
            info!("Generating the server authority certificate...");

            // Create certificate directory
            fs::create_dir_all(&self.certificate_path).await?;

            // Generate self-signed CA certificate
            self.create_certificate(
                "woodstock.shadoware.org",
                false,
                &cert_path,
                &key_path,
                None, // No issuer = self-signed
            )
            .await?;
        }

        Ok(())
    }

    /// Generate HTTPS certificate for the server (https.pem, https.key)
    pub async fn generate_https_certificate(&self) -> Result<()> {
        let https_cert_path = self.certificate_path.join("https.pem");
        let https_key_path = self.certificate_path.join("https.key");

        if !self
            .cert_files_exist(&https_cert_path, &https_key_path)
            .await
        {
            info!("Generating the HTTPS certificate...");

            // Ensure CA exists first
            self.generate_certificate().await?;

            // Load CA issuer for signing
            let issuer = self.load_ca_issuer().await?;

            // Generate HTTPS certificate using specialized method (matching TypeScript)
            self.create_https_certificate(
                "localhost", // Using "localhost" as default hostname
                &https_cert_path,
                &https_key_path,
                &issuer,
            )
            .await?;
        }

        Ok(())
    }

    /// Generate server certificate for host communication ({host}_client.pem/key)
    async fn generate_host_server_certificate(&self, hostname: &str) -> Result<()> {
        let cert_path = self
            .certificate_path
            .join(format!("{}_client.pem", hostname));
        let key_path = self
            .certificate_path
            .join(format!("{}_client.key", hostname));

        if !self.cert_files_exist(&cert_path, &key_path).await {
            info!("Generating server host {} certificate...", hostname);

            // Ensure CA exists first
            self.generate_certificate().await?;

            // Load CA issuer for signing
            let issuer = self.load_ca_issuer().await?;

            // Generate host server certificate signed by root CA (not server auth)
            self.create_certificate(
                hostname,
                false, // is_server = false for client authentication
                &cert_path,
                &key_path,
                Some(&issuer),
            )
            .await?;
        }

        Ok(())
    }

    /// Generate client authority certificate for host ({host}_ca.pem/key)
    /// This creates a SELF-SIGNED CA certificate, NOT signed by root CA
    async fn generate_client_authority_certificate(&self, hostname: &str) -> Result<()> {
        let cert_path = self.certificate_path.join(format!("{}_ca.pem", hostname));
        let key_path = self.certificate_path.join(format!("{}_ca.key", hostname));

        if !self.cert_files_exist(&cert_path, &key_path).await {
            info!("Generating the client authority certificate...");

            // Create self-signed CA certificate for this host (matching TypeScript)
            // TypeScript: this.#createCertificate(host + '.woodstock.shadoware.org', true);
            let ca_hostname = format!("{}.woodstock.shadoware.org", hostname);

            // Create CA certificate parameters
            let mut params = CertificateParams::new(vec![ca_hostname.clone()])?;
            params.distinguished_name = Self::create_certificate_attributes(&ca_hostname);
            params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
            params.key_usages = vec![KeyUsagePurpose::KeyCertSign];

            // Set validity period (10 years to match TypeScript)
            let now = OffsetDateTime::now_utc();
            params.not_before = now;
            params.not_after = now + Duration::days(365 * 10);

            // Generate key pair and self-sign
            let key_pair = KeyPair::generate()?;
            let cert = params.self_signed(&key_pair)?;

            // Write certificate and key
            fs::write(&cert_path, cert.pem()).await?;
            fs::write(&key_path, key_pair.serialize_pem()).await?;
        }

        Ok(())
    }

    /// Load host CA certificate and create issuer for signing certificates
    pub async fn load_host_ca_issuer(&self, hostname: &str) -> Result<Issuer<'static, KeyPair>> {
        let cert_path = self.certificate_path.join(format!("{}_ca.pem", hostname));
        let key_path = self.certificate_path.join(format!("{}_ca.key", hostname));

        // Ensure host CA exists
        if !self.cert_files_exist(&cert_path, &key_path).await {
            return Err(eyre!(
                "Host CA does not exist for {}. Generate it first.",
                hostname
            ));
        }

        // Read the key pair
        let key_pem = fs::read_to_string(&key_path).await?;
        let key_pair = KeyPair::from_pem(&key_pem)?;

        // Read and parse existing certificate to extract params
        let cert_pem = fs::read_to_string(&cert_path).await?;
        let ca_params = self.parse_certificate_params(&cert_pem)?;

        // Create issuer from parsed host CA params
        Ok(Issuer::new(ca_params, key_pair))
    }

    /// Generate client certificate signed by host CA ({host}_server.pem/key)
    async fn generate_host_client_certificate(&self, hostname: &str) -> Result<()> {
        let cert_path = self
            .certificate_path
            .join(format!("{}_server.pem", hostname));
        let key_path = self
            .certificate_path
            .join(format!("{}_server.key", hostname));

        if !self.cert_files_exist(&cert_path, &key_path).await {
            info!("Generating client host {} certificate...", hostname);

            // Ensure host CA exists first
            self.generate_client_authority_certificate(hostname).await?;

            // Load host CA issuer for signing
            let issuer = self.load_host_ca_issuer(hostname).await?;

            // Generate host client certificate signed by host CA
            // TypeScript: this.#createCertificate(host, false, clientCA);
            self.create_certificate(
                hostname,
                false, // is_server = false (client auth, not server auth)
                &cert_path,
                &key_path,
                Some(&issuer),
            )
            .await?;
        }

        Ok(())
    }

    /// Generate host HTTPS certificate signed by root CA ({host}_https.pem/key)
    async fn generate_host_https_certificate(&self, hostname: &str) -> Result<()> {
        let cert_path = self
            .certificate_path
            .join(format!("{}_https.pem", hostname));
        let key_path = self
            .certificate_path
            .join(format!("{}_https.key", hostname));

        if !self.cert_files_exist(&cert_path, &key_path).await {
            info!("Generating https host {} certificate...", hostname);

            // Ensure root CA exists first
            self.generate_certificate().await?;

            // Load root CA issuer for signing
            let issuer = self.load_ca_issuer().await?;

            // Generate host HTTPS certificate using specialized method (matching TypeScript)
            self.create_https_certificate(hostname, &cert_path, &key_path, &issuer)
                .await?;
        }

        Ok(())
    }

    /// Generate all certificates for a specific host
    pub async fn generate_host_certificate(&self, hostname: &str) -> Result<()> {
        // Generate all required certificates for the host
        self.generate_host_server_certificate(hostname).await?;
        self.generate_client_authority_certificate(hostname).await?;
        self.generate_host_client_certificate(hostname).await?;
        self.generate_host_https_certificate(hostname).await?;

        Ok(())
    }

    /// Parse certificate parameters from existing PEM certificate using x509-parser
    /// This extracts the key information and assumes standard configuration for the rest
    pub fn parse_certificate_params(&self, cert_pem: &str) -> Result<CertificateParams> {
        // Parse PEM to DER
        let cert_der = CertificateDer::from_pem_slice(cert_pem.as_bytes())
            .map_err(|e| eyre!("Failed to parse PEM certificate: {}", e))?;

        // Use x509-parser to extract certificate information
        let (_remainder, x509) = x509_parser::parse_x509_certificate(&cert_der)
            .map_err(|_| eyre!("Failed to parse X.509 certificate"))?;

        // Extract and parse subject common name to determine hostname
        let subject_cn = x509
            .tbs_certificate
            .subject
            .iter_common_name()
            .next()
            .and_then(|attr| attr.as_str().ok())
            .unwrap_or("unknown");

        // Create new params with extracted hostname
        let mut params = CertificateParams::new(vec![subject_cn.to_string()])?;

        // Set distinguished name using our standard method
        params.distinguished_name = Self::create_certificate_attributes(subject_cn);

        // Extract validity period
        params.not_before = x509.validity().not_before.to_datetime();
        params.not_after = x509.validity().not_after.to_datetime();

        // Extract serial number
        params.serial_number = Some(x509.serial.to_bytes_be().into());

        // Determine if this is a CA certificate by checking basic constraints
        let is_ca = x509
            .basic_constraints()
            .ok()
            .flatten()
            .map(|bc| bc.value.ca)
            .unwrap_or(false);

        if is_ca {
            params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
            params.key_usages = vec![KeyUsagePurpose::KeyCertSign];
        } else {
            params.is_ca = IsCa::NoCa;
            // For end-entity certificates, determine usage based on extensions
            params.key_usages = vec![
                KeyUsagePurpose::KeyEncipherment,
                KeyUsagePurpose::DigitalSignature,
            ];
            params.extended_key_usages = vec![
                ExtendedKeyUsagePurpose::ServerAuth,
                ExtendedKeyUsagePurpose::ClientAuth,
            ];
        }

        Ok(params)
    }

    /// Get the certificate path
    pub fn certificate_path(&self) -> &Path {
        &self.certificate_path
    }
}
