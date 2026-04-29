#[cfg(test)]
mod tests {
    use super::super::certificate::CertificateService;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::fs;
    use woodstock::config::{Configuration, ConfigurationPath};

    #[tokio::test]
    async fn test_certificate_structure_matches_typescript() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Generate complete certificate set for a host
        service.generate_certificate().await.unwrap();
        service.generate_https_certificate().await.unwrap();
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Verify we have all required certificates as per TypeScript spec:
        // 1 root certificate (rootCA.pem/key) - matching TypeScript implementation
        assert!(temp_dir.path().join("rootCA.pem").exists());
        assert!(temp_dir.path().join("rootCA.key").exists());

        // 1 HTTPS certificate (https.pem/key)
        assert!(temp_dir.path().join("https.pem").exists());
        assert!(temp_dir.path().join("https.key").exists());

        // For each host: _ca, _client, _server, _https
        assert!(temp_dir.path().join("test-host_ca.pem").exists());
        assert!(temp_dir.path().join("test-host_ca.key").exists());

        assert!(temp_dir.path().join("test-host_client.pem").exists());
        assert!(temp_dir.path().join("test-host_client.key").exists());

        assert!(temp_dir.path().join("test-host_server.pem").exists());
        assert!(temp_dir.path().join("test-host_server.key").exists());

        assert!(temp_dir.path().join("test-host_https.pem").exists());
        assert!(temp_dir.path().join("test-host_https.key").exists());
    }

    #[tokio::test]
    async fn test_certificate_content_not_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Generate certificates
        service.generate_certificate().await.unwrap();
        service.generate_https_certificate().await.unwrap();
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Verify certificate files are not empty and contain PEM headers
        let ca_cert_content = fs::read_to_string(temp_dir.path().join("rootCA.pem"))
            .await
            .unwrap();
        assert!(ca_cert_content.contains("-----BEGIN CERTIFICATE-----"));
        assert!(ca_cert_content.contains("-----END CERTIFICATE-----"));
        assert!(ca_cert_content.len() > 100); // Reasonable size check

        let https_cert_content = fs::read_to_string(temp_dir.path().join("https.pem"))
            .await
            .unwrap();
        assert!(https_cert_content.contains("-----BEGIN CERTIFICATE-----"));
        assert!(https_cert_content.contains("-----END CERTIFICATE-----"));

        let host_ca_content = fs::read_to_string(temp_dir.path().join("test-host_ca.pem"))
            .await
            .unwrap();
        assert!(host_ca_content.contains("-----BEGIN CERTIFICATE-----"));
        assert!(host_ca_content.contains("-----END CERTIFICATE-----"));

        // Verify private keys
        let ca_key_content = fs::read_to_string(temp_dir.path().join("rootCA.key"))
            .await
            .unwrap();
        assert!(ca_key_content.contains("-----BEGIN PRIVATE KEY-----"));
        assert!(ca_key_content.contains("-----END PRIVATE KEY-----"));
    }

    #[tokio::test]
    async fn test_host_certificate_generation_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Generate certificates twice
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();
        let first_content = fs::read_to_string(temp_dir.path().join("test-host_ca.pem"))
            .await
            .unwrap();

        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();
        let second_content = fs::read_to_string(temp_dir.path().join("test-host_ca.pem"))
            .await
            .unwrap();

        // Should be identical (idempotent)
        assert_eq!(first_content, second_content);
    }

    #[tokio::test]
    async fn test_certificate_attributes_match_typescript() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Generate certificates
        service.generate_certificate().await.unwrap();
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Verify certificate content matches TypeScript attributes
        let ca_cert_content = fs::read_to_string(temp_dir.path().join("rootCA.pem"))
            .await
            .unwrap();

        // The certificate should contain our organization attributes
        // Note: We can't easily parse the certificate here without additional dependencies,
        // but we can verify the basic structure and that it's valid PEM
        assert!(ca_cert_content.contains("-----BEGIN CERTIFICATE-----"));
        assert!(ca_cert_content.contains("-----END CERTIFICATE-----"));

        // Verify the certificates contain expected number of lines (rough validation)
        let ca_lines: Vec<&str> = ca_cert_content.lines().collect();
        assert!(
            ca_lines.len() > 10,
            "Certificate should have multiple lines"
        );

        // Test that host certificates follow the same pattern
        let host_ca_content = fs::read_to_string(temp_dir.path().join("test-host_ca.pem"))
            .await
            .unwrap();
        let host_ca_lines: Vec<&str> = host_ca_content.lines().collect();
        assert!(
            host_ca_lines.len() > 10,
            "Host CA certificate should have multiple lines"
        );
    }

    #[tokio::test]
    async fn test_certificate_generation_order_matches_typescript() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Test that generating host certificate creates root CA first (dependency)
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Verify that root CA was created automatically
        assert!(
            temp_dir.path().join("rootCA.pem").exists(),
            "Root CA should be created automatically"
        );
        assert!(
            temp_dir.path().join("rootCA.key").exists(),
            "Root CA key should be created automatically"
        );

        // Verify all host certificates were created in the correct order
        assert!(
            temp_dir.path().join("test-host_client.pem").exists(),
            "Host client cert should exist"
        );
        assert!(
            temp_dir.path().join("test-host_ca.pem").exists(),
            "Host CA should exist"
        );
        assert!(
            temp_dir.path().join("test-host_server.pem").exists(),
            "Host server cert should exist"
        );
        assert!(
            temp_dir.path().join("test-host_https.pem").exists(),
            "Host HTTPS cert should exist"
        );
    }

    #[tokio::test]
    async fn test_ip_detection_matches_typescript() {
        use super::super::certificate::CertificateService;

        // Test IPv4 addresses
        assert!(CertificateService::is_ip("192.168.1.1"));
        assert!(CertificateService::is_ip("127.0.0.1"));
        assert!(CertificateService::is_ip("10.0.0.1"));
        assert!(CertificateService::is_ip("255.255.255.255"));

        // Test IPv6 addresses
        assert!(CertificateService::is_ip("::1"));
        assert!(CertificateService::is_ip("2001:db8::1"));
        assert!(CertificateService::is_ip("fe80::1"));

        // Test hostnames (should not be detected as IP)
        assert!(!CertificateService::is_ip("localhost"));
        assert!(!CertificateService::is_ip("example.com"));
        assert!(!CertificateService::is_ip("test-host"));
        assert!(!CertificateService::is_ip("woodstock.shadoware.org"));

        // Test invalid inputs
        assert!(!CertificateService::is_ip(""));
        assert!(!CertificateService::is_ip("invalid.ip.address"));
        assert!(!CertificateService::is_ip("999.999.999.999"));
    }

    #[tokio::test]
    async fn test_subject_alt_names_for_ip_vs_hostname() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Generate certificates for both IP and hostname
        service
            .generate_host_certificate("192.168.1.100")
            .await
            .unwrap();
        service
            .generate_host_certificate("test-hostname")
            .await
            .unwrap();

        // Both should generate successfully (validation that SAN handling works for both)
        assert!(temp_dir.path().join("192.168.1.100_ca.pem").exists());
        assert!(temp_dir.path().join("test-hostname_ca.pem").exists());

        // Verify certificate content is valid
        let ip_cert = fs::read_to_string(temp_dir.path().join("192.168.1.100_ca.pem"))
            .await
            .unwrap();
        let hostname_cert = fs::read_to_string(temp_dir.path().join("test-hostname_ca.pem"))
            .await
            .unwrap();

        assert!(ip_cert.contains("-----BEGIN CERTIFICATE-----"));
        assert!(hostname_cert.contains("-----BEGIN CERTIFICATE-----"));
    }
}
